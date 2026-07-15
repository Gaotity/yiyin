import { join } from 'node:path'

import { tryCatch } from '@utils'
import { app } from 'electron'

const env = import.meta.env
const isPackaged = app.isPackaged

export const appPath = app.getAppPath()
export const userDataPath = tryCatch(() => app.getPath('userData'), appPath) ?? appPath
export const desktopPath = tryCatch(() => app.getPath('desktop'), userDataPath) ?? userDataPath
export function getPath(name: Parameters<typeof app.getPath>[0]) {
  return tryCatch(() => app.getPath(name), desktopPath, () => {}) ?? desktopPath
}

const paths = {
  preload: join(isPackaged ? app.getAppPath() : (env.VITE_DIST_ELECTRON ?? app.getAppPath()), 'preload/index.cjs'),
  web: isPackaged ? join(app.getAppPath(), 'web') : (env.VITE_WEB ?? join(app.getAppPath(), 'web')),
  logger: join(userDataPath, 'logs'),
}

export default paths
