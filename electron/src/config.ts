import type { ConfigMutation } from '@common/models/config'
import type { IConfig, IFieldInfoItem } from '@src/interface'
import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { migrateConfig } from '@root/config/migrate'
import { createDefaultConfig } from '@root/config/model'
import { getPath, userDataPath } from './path'

export const DefaultConfig = createDefaultConfig(
  userDataPath,
  getPath('pictures'),
  getPath('temp'),
  import.meta.env.VITE_VERSION ?? '1.6.0',
)

function ensureDirectories(value: IConfig) {
  for (const directory of [value.output, value.cacheDir, value.staticDir, value.font.dir]) {
    fs.mkdirSync(directory, { recursive: true })
  }
}

function writeAtomically(value: IConfig) {
  fs.mkdirSync(path.dirname(value.dir), { recursive: true })
  if (fs.existsSync(value.dir)) fs.copyFileSync(value.dir, `${value.dir}.bak`)
  const temporaryPath = `${value.dir}.${process.pid}.tmp`
  fs.writeFileSync(temporaryPath, JSON.stringify(value, null, 2), { encoding: 'utf8', mode: 0o600 })
  fs.renameSync(temporaryPath, value.dir)
}

function readConfig() {
  const defaults = structuredClone(DefaultConfig)
  if (!fs.existsSync(defaults.dir)) {
    ensureDirectories(defaults)
    return defaults
  }

  try {
    const raw = JSON.parse(fs.readFileSync(defaults.dir, 'utf8')) as unknown
    const migrated = migrateConfig(raw, defaults)
    ensureDirectories(migrated)
    if (JSON.stringify(raw) !== JSON.stringify(migrated)) writeAtomically(migrated)
    return migrated
  }
  catch {
    fs.copyFileSync(defaults.dir, `${defaults.dir}.invalid.bak`)
    ensureDirectories(defaults)
    writeAtomically(defaults)
    return defaults
  }
}

export const config: IConfig = readConfig()

export function getConfig(useDefaults = false) {
  return structuredClone(useDefaults ? DefaultConfig : config)
}

export function storeConfig(update: Partial<IConfig>) {
  Object.assign(config, update)
  ensureDirectories(config)
  writeAtomically(config)
  return config
}

export function storePublicConfig(mutation: ConfigMutation, resolveResourceUrl: (url: string) => string) {
  const hydrateField = (field: IFieldInfoItem<string | number | boolean>): IFieldInfoItem<string | number | boolean> => ({
    ...field,
    bImg: field.bImg ? resolveResourceUrl(field.bImg) : '',
    wImg: field.wImg ? resolveResourceUrl(field.wImg) : '',
  })

  return storeConfig({
    options: mutation.options,
    tempFields: mutation.tempFields.map(hydrateField),
    customTempFields: mutation.customTempFields.map(hydrateField),
    temps: mutation.temps,
  })
}
