import { describe, expect, it } from 'vitest'
import { migrateConfig } from '../../electron/config/migrate'
import { createDefaultConfig } from '../../electron/config/model'

describe('configuration migration', () => {
  const defaults = createDefaultConfig('/user', '/pictures', '/tmp')

  it('adds the schema version while preserving safe user options', () => {
    const migrated = migrateConfig({
      version: '1.6.0',
      output: '/chosen',
      options: { quality: 80 },
    }, defaults)

    expect(migrated.schemaVersion).toBe(1)
    expect(migrated.output).toBe('/chosen')
    expect(migrated.options.quality).toBe(80)
    expect(migrated.options.main_img_w_rate).toBe(90)
  })

  it('falls back to defaults for invalid or future configuration', () => {
    expect(migrateConfig('invalid', defaults)).toEqual(defaults)
    expect(migrateConfig({ schemaVersion: 999 }, defaults)).toEqual(defaults)
    expect(migrateConfig({ options: { quality: 1_000 }, temps: [{ use: 'yes' }] }, defaults).options).toEqual(defaults.options)
    expect(migrateConfig({ options: { quality: 1_000 }, temps: [{ use: 'yes' }] }, defaults).temps).toEqual(defaults.temps)
  })
})
