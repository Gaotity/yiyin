import type { IConfig } from '../src/interface'
import path from 'node:path'
import { configMutationSchema } from '@common/config/schema'

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

export function migrateConfig(input: unknown, defaults: IConfig): IConfig {
  if (!isRecord(input)) return structuredClone(defaults)
  if (typeof input.schemaVersion === 'number' && input.schemaVersion > defaults.schemaVersion) return structuredClone(defaults)

  const migrated = structuredClone(defaults)
  if (typeof input.version === 'string' && input.version.length <= 100) migrated.version = input.version
  if (typeof input.output === 'string' && input.output.length <= 4_096 && path.isAbsolute(input.output)) migrated.output = input.output

  const mutation = configMutationSchema.safeParse({
    options: isRecord(input.options) ? { ...migrated.options, ...input.options } : migrated.options,
    tempFields: Array.isArray(input.tempFields) ? input.tempFields : migrated.tempFields,
    customTempFields: Array.isArray(input.customTempFields) ? input.customTempFields : migrated.customTempFields,
    temps: Array.isArray(input.temps) ? input.temps : migrated.temps,
  })
  if (mutation.success) {
    migrated.options = mutation.data.options
    migrated.tempFields = mutation.data.tempFields
    migrated.customTempFields = mutation.data.customTempFields
    migrated.temps = mutation.data.temps
  }

  if (isRecord(input.font) && isRecord(input.font.map)) {
    const fontMap: Record<string, string> = {}
    for (const [name, fileName] of Object.entries(input.font.map)) {
      if (name.length <= 100 && typeof fileName === 'string' && fileName.length <= 255 && path.basename(fileName) === fileName) {
        fontMap[name] = fileName
      }
    }
    migrated.font.map = fontMap
  }
  migrated.schemaVersion = defaults.schemaVersion
  return migrated
}
