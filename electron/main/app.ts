import type { BrowserWindow, BrowserWindowConstructorOptions } from 'electron'
import process from 'node:process'
import { Logger } from '@modules/logger'
import { ipcChannels } from '@root/ipc/channels'
import { registerIpcHandlers } from '@root/ipc/register'
import { registerYiyinProtocol } from '@root/main/protocol'
import { resourceRegistry } from '@root/resources'
import { genMainImgShadowQueue, genTextImgQueue, imageToolQueue } from '@src/common/queue'
import paths from '@src/path'
import { app } from 'electron'
import { createWindow } from './create-window'

const log = new Logger('App')

export default class Application {
  win!: BrowserWindow
  private initialized = false

  async init() {
    app.enableSandbox()
    if (process.platform === 'win32') app.setAppUserModelId('io.github.gaotity.yiyin')

    if (!app.requestSingleInstanceLock()) {
      app.quit()
      return
    }

    app.on('window-all-closed', () => {
      if (process.platform !== 'darwin') app.quit()
    })
    app.on('second-instance', () => {
      if (this.win?.isMinimized()) this.win.restore()
      this.win?.focus()
    })
    app.on('before-quit', () => {
      void genMainImgShadowQueue.close()
      void genTextImgQueue.close()
      void imageToolQueue.close()
    })

    await app.whenReady()
    registerYiyinProtocol(paths.web, resourceRegistry)
    this.initialized = true
  }

  async start() {
    if (!this.initialized) throw new Error('Application must be initialized before start')
    await this.createDefaultWindow()
    registerIpcHandlers(this.win, resourceRegistry)

    imageToolQueue.on(async (imageTool) => {
      try {
        await imageTool.genWatermark()
      }
      catch (error) {
        log.error('Image task failed', error)
        this.win.webContents.send(ipcChannels.events.failure, {
          taskId: imageTool.id,
          message: 'Image processing failed',
        })
      }
    })
    imageToolQueue.onerror((type, error) => log.error('Image queue failure [%s]', type, error))
  }

  private async createDefaultWindow() {
    const options: BrowserWindowConstructorOptions = {
      width: 900 + (import.meta.env.DEV ? 500 : 0),
      height: 730,
      title: '壹印',
      frame: false,
      show: false,
    }
    if (import.meta.env.PROD) {
      options.minWidth = options.width
      options.minHeight = options.height
      options.maxWidth = options.width
      options.maxHeight = options.height
    }

    this.win = await createWindow('main', options)
    this.win.once('ready-to-show', () => this.win.show())
    this.win.on('closed', () => app.quit())
  }
}
