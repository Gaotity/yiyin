import type { PlatformResult } from '@common/platform/result'
import type { BrowserWindow, IpcMainInvokeEvent } from 'electron'
import type { ZodType } from 'zod'
import type { ResourceRegistry } from '../resources/registry'
import fs from 'node:fs'
import path from 'node:path'
import { app, dialog, ipcMain, shell } from 'electron'
import { ZodError } from 'zod'
import { getPublicConfig } from '../config/public'
import { assertBatchSize } from '../resources/policy'
import { genMainImgShadowQueue, genTextImgQueue, imageToolQueue } from '../src/common/queue'
import { config, getConfig, storeConfig, storePublicConfig } from '../src/config'
import { ExifReaderService } from '../src/modules/exif-reader'
import { ImageTool } from '../src/modules/image-tool'
import { ipcChannels } from './channels'
import { failure, PlatformBoundaryError, success } from './errors'
import { ipcSchemas } from './schemas'
import { assertTrustedSender } from './sender'

type Handler<T, R> = (input: T) => R | Promise<R>

function register<T, R>(channel: string, schema: ZodType<T>, handler: Handler<T, R>) {
  ipcMain.handle(channel, async (event: IpcMainInvokeEvent, raw: unknown): Promise<PlatformResult<R>> => {
    try {
      assertTrustedSender(event, import.meta.env.DEV)
      const input = schema.parse(raw)
      return success(await handler(input))
    }
    catch (error) {
      if (error instanceof ZodError) return failure(new PlatformBoundaryError('INVALID_REQUEST', 'The request payload is invalid'))
      console.error(`IPC handler failed: ${channel}`, error)
      return failure(error)
    }
  })
}

export function registerIpcHandlers(window: BrowserWindow, registry: ResourceRegistry) {
  register(ipcChannels.app.minimize, ipcSchemas.empty, () => {
    window.minimize()
    return true as const
  })
  register(ipcChannels.app.close, ipcSchemas.empty, () => {
    setImmediate(() => app.quit())
    return true as const
  })

  register(ipcChannels.config.get, ipcSchemas.empty, () => getPublicConfig(registry))
  register(ipcChannels.config.update, ipcSchemas.configMutation, async (mutation) => {
    storePublicConfig(mutation, url => registry.resolveUrl(url))
    return getPublicConfig(registry)
  })
  register(ipcChannels.config.reset, ipcSchemas.empty, async () => {
    storeConfig(getConfig(true))
    return getPublicConfig(registry)
  })
  register(ipcChannels.config.chooseOutputDirectory, ipcSchemas.empty, async () => {
    const result = await dialog.showOpenDialog(window, { properties: ['openDirectory'] })
    if (result.canceled || !result.filePaths[0]) return null
    storeConfig({ output: result.filePaths[0], cacheDir: import.meta.env.DEV ? path.join(result.filePaths[0], '.cache') : config.cacheDir })
    return getPublicConfig(registry)
  })
  register(ipcChannels.config.openOutputDirectory, ipcSchemas.empty, async () => {
    const error = await shell.openPath(config.output)
    if (error) throw new PlatformBoundaryError('INTERNAL', 'The output directory could not be opened')
    return true as const
  })

  register(ipcChannels.files.registerImages, ipcSchemas.imageFiles, async ({ files }) => {
    assertBatchSize(files.length)
    const tasks = []
    for (const file of files) {
      const descriptor = await registry.registerImage(file.path, file.name)
      const tool = new ImageTool(file.path, descriptor.displayName, {
        cachePath: config.cacheDir,
        outputOption: structuredClone(config.options),
        outputPath: config.output,
      }, descriptor.resourceId)
      tool.on('progress', (taskId, progress) => window.webContents.send(ipcChannels.events.progress, { taskId, progress }))
      imageToolQueue.add(tool, descriptor.resourceId)
      tasks.push({ ...descriptor, taskId: descriptor.resourceId })
    }
    return tasks
  })

  register(ipcChannels.files.registerFont, ipcSchemas.font, async ({ file, name }) => {
    if (config.font.map[name]) throw new PlatformBoundaryError('FILE_INVALID', 'A font with this name already exists')
    const source = await registry.registerFont(file.path, file.name)
    const sourcePath = registry.resolve(source.resourceId)
    const fileName = `${source.resourceId}${path.extname(sourcePath).toLowerCase()}`
    const targetPath = path.join(config.font.dir, fileName)
    fs.copyFileSync(sourcePath, targetPath)
    config.font.map[name] = fileName
    storeConfig({ font: config.font })
    const descriptor = await registry.registerManaged(targetPath, fileName, 'font')
    return { ...descriptor, name }
  })

  register(ipcChannels.files.removeFont, ipcSchemas.removeFont, ({ name }) => {
    const fileName = config.font.map[name]
    if (!fileName) return false
    const filePath = path.join(config.font.dir, fileName)
    if (fs.existsSync(filePath)) fs.rmSync(filePath)
    delete config.font.map[name]
    storeConfig({ font: config.font })
    return true
  })

  register(ipcChannels.files.registerOverlay, ipcSchemas.overlay, async ({ file, slot }) => {
    const source = await registry.registerImage(file.path, file.name)
    const sourcePath = registry.resolve(source.resourceId)
    const targetPath = path.join(config.staticDir, `${slot}-${source.resourceId}${path.extname(sourcePath).toLowerCase()}`)
    fs.copyFileSync(sourcePath, targetPath)
    return registry.registerManaged(targetPath, file.name, 'image')
  })

  register(ipcChannels.tasks.start, ipcSchemas.empty, () => {
    void imageToolQueue.run()
    return true as const
  })
  register(ipcChannels.tasks.preview, ipcSchemas.taskId, async ({ taskId }) => {
    const filePath = registry.resolve(taskId)
    const descriptor = registry.descriptor(taskId)
    const tool = new ImageTool(filePath, descriptor.displayName, {
      cachePath: config.cacheDir,
      outputOption: structuredClone(config.options),
      outputPath: config.output,
    }, taskId)
    const preview = await tool.genPreview()
    if (!preview) throw new PlatformBoundaryError('INTERNAL', 'The preview could not be generated')
    return preview
  })
  register(ipcChannels.tasks.readExif, ipcSchemas.taskId, ({ taskId }) => new ExifReaderService(registry.resolve(taskId)).parse())
  register(ipcChannels.tasks.clear, ipcSchemas.empty, () => {
    imageToolQueue.drain()
    return true as const
  })
  register(ipcChannels.tasks.completeTextRender, ipcSchemas.textRender, ({ taskId, images }) => {
    genTextImgQueue.add({ id: taskId, textImgList: images })
    return true as const
  })
  register(ipcChannels.tasks.completeShadowRender, ipcSchemas.shadowRender, ({ taskId, dataUrl }) => {
    genMainImgShadowQueue.add({ id: taskId, data: dataUrl })
    return true as const
  })
}
