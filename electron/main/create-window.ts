import type { BrowserWindowConstructorOptions } from 'electron'
import paths from '@src/path'
import { BrowserWindow, shell } from 'electron'
import { isAllowedExternalUrl, isTrustedRendererUrl } from '../security/urls'

export async function createWindow(route: string, options: BrowserWindowConstructorOptions) {
  const window = new BrowserWindow({
    ...options,
    webPreferences: {
      preload: paths.preload,
      nodeIntegration: false,
      contextIsolation: true,
      sandbox: true,
      webSecurity: true,
      allowRunningInsecureContent: false,
      spellcheck: false,
    },
  })

  window.webContents.setWindowOpenHandler(({ url }) => {
    if (isAllowedExternalUrl(url)) void shell.openExternal(url)
    return { action: 'deny' }
  })
  window.webContents.on('will-navigate', (event, url) => {
    if (!isTrustedRendererUrl(url, import.meta.env.DEV)) event.preventDefault()
  })
  window.webContents.session.setPermissionCheckHandler(() => false)
  window.webContents.session.setPermissionRequestHandler((_webContents, _permission, callback) => callback(false))

  if (import.meta.env.DEV) {
    await window.loadURL(`http://127.0.0.1:5173/${route}/index.html`)
  }
  else {
    await window.loadURL(`yiyin://app/${route}/index.html`)
  }

  return window
}
