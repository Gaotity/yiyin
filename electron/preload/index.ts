import type { PlatformBridge, PlatformFile } from '@common/platform/bridge'
import { contextBridge, ipcRenderer, webUtils } from 'electron'
import { ipcChannels } from '../ipc/channels'

function candidate(file: PlatformFile) {
  return {
    name: file.name,
    path: webUtils.getPathForFile(file as never),
  }
}

function subscribe<T>(channel: string, listener: (event: T) => void) {
  const wrapped = (_event: Electron.IpcRendererEvent, payload: T) => listener(payload)
  ipcRenderer.on(channel, wrapped)
  return () => ipcRenderer.removeListener(channel, wrapped)
}

const platform: PlatformBridge = {
  app: {
    minimize: () => ipcRenderer.invoke(ipcChannels.app.minimize),
    close: () => ipcRenderer.invoke(ipcChannels.app.close),
  },
  config: {
    get: () => ipcRenderer.invoke(ipcChannels.config.get),
    update: mutation => ipcRenderer.invoke(ipcChannels.config.update, mutation),
    reset: () => ipcRenderer.invoke(ipcChannels.config.reset),
    chooseOutputDirectory: () => ipcRenderer.invoke(ipcChannels.config.chooseOutputDirectory),
    openOutputDirectory: () => ipcRenderer.invoke(ipcChannels.config.openOutputDirectory),
  },
  files: {
    registerImages: files => ipcRenderer.invoke(ipcChannels.files.registerImages, { files: files.map(candidate) }),
    registerFont: (file, name) => ipcRenderer.invoke(ipcChannels.files.registerFont, { file: candidate(file), name }),
    removeFont: name => ipcRenderer.invoke(ipcChannels.files.removeFont, { name }),
    registerOverlay: (file, slot) => ipcRenderer.invoke(ipcChannels.files.registerOverlay, { file: candidate(file), slot }),
  },
  tasks: {
    start: () => ipcRenderer.invoke(ipcChannels.tasks.start),
    preview: taskId => ipcRenderer.invoke(ipcChannels.tasks.preview, { taskId }),
    readExif: taskId => ipcRenderer.invoke(ipcChannels.tasks.readExif, { taskId }),
    clear: () => ipcRenderer.invoke(ipcChannels.tasks.clear),
    completeTextRender: (taskId, images) => ipcRenderer.invoke(ipcChannels.tasks.completeTextRender, { taskId, images }),
    completeShadowRender: (taskId, dataUrl) => ipcRenderer.invoke(ipcChannels.tasks.completeShadowRender, { taskId, dataUrl }),
  },
  events: {
    onProgress: listener => subscribe(ipcChannels.events.progress, listener),
    onFailure: listener => subscribe(ipcChannels.events.failure, listener),
    onTextRender: listener => subscribe(ipcChannels.events.textRender, listener),
    onShadowRender: listener => subscribe(ipcChannels.events.shadowRender, listener),
  },
}

contextBridge.exposeInMainWorld('platform', Object.freeze(platform))
