import fs from 'node:fs'
import path from 'node:path'
import { format } from 'node:util'

type LogLevel = 'DEBUG' | 'ERROR' | 'FATAL' | 'INFO' | 'OFF' | 'WARN'
type ExportMode = 'CONSOLE' | 'CONSOLE_FILE' | 'FILE'

interface LoggerConfig {
  exportMode?: ExportMode
  level?: LogLevel
  namespace?: string
  path?: string
}

const levelOrder: Record<LogLevel, number> = {
  DEBUG: 10,
  INFO: 20,
  WARN: 30,
  ERROR: 40,
  FATAL: 50,
  OFF: Number.POSITIVE_INFINITY,
}

let config: Required<LoggerConfig> = {
  exportMode: 'CONSOLE',
  level: 'INFO',
  namespace: 'main',
  path: '',
}

export function setLoggerConfig(next: LoggerConfig) {
  config = { ...config, ...next }
  if (config.path && config.exportMode !== 'CONSOLE') {
    fs.mkdirSync(config.path, { recursive: true, mode: 0o700 })
  }
}

export class Logger {
  constructor(private readonly namespace = config.namespace) {}

  debug(message?: unknown, ...values: unknown[]) {
    return this.write('DEBUG', message, values)
  }

  info(message?: unknown, ...values: unknown[]) {
    return this.write('INFO', message, values)
  }

  warn(message?: unknown, ...values: unknown[]) {
    return this.write('WARN', message, values)
  }

  error(message?: unknown, ...values: unknown[]) {
    return this.write('ERROR', message, values)
  }

  fatal(message?: unknown, ...values: unknown[]) {
    return this.write('FATAL', message, values)
  }

  private write(level: LogLevel, message: unknown, values: unknown[]) {
    if (levelOrder[level] < levelOrder[config.level]) return this

    const line = `${new Date().toISOString()} [${level}] [${this.namespace}] ${format(message, ...values)}`
    if (config.exportMode !== 'FILE') console[level === 'DEBUG' ? 'debug' : level === 'INFO' ? 'info' : 'error'](line)
    if (config.exportMode !== 'CONSOLE' && config.path) {
      try {
        fs.appendFileSync(path.join(config.path, 'application.log'), `${line}\n`, { encoding: 'utf8', mode: 0o600 })
      }
      catch (error) {
        console.error('Unable to persist application log', error)
      }
    }
    return this
  }
}

export async function closeAllLogger() {}
