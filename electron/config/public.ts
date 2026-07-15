import type { FieldInfoItem, FieldValue, PublicConfig } from '@common/models/config'
import type { ResourceRegistry } from '../resources/registry'
import fs from 'node:fs'
import path from 'node:path'
import { config } from '../src/config'

async function publicField(field: FieldInfoItem<FieldValue>, registry: ResourceRegistry): Promise<FieldInfoItem<FieldValue>> {
  const resourceUrl = async (filePath: string) => {
    if (!filePath) return ''
    if (filePath.startsWith('yiyin://resource/')) return filePath
    if (!fs.existsSync(filePath)) return ''
    return (await registry.registerManaged(filePath, path.basename(filePath), 'image')).resourceUrl
  }

  return {
    ...field,
    bImg: await resourceUrl(field.bImg),
    wImg: await resourceUrl(field.wImg),
  }
}

export async function getPublicConfig(registry: ResourceRegistry): Promise<PublicConfig> {
  const fonts = []
  for (const [name, fileName] of Object.entries(config.font.map)) {
    const filePath = path.join(config.font.dir, fileName)
    if (!fs.existsSync(filePath)) continue
    const descriptor = await registry.registerManaged(filePath, fileName, 'font')
    fonts.push({ ...descriptor, name })
  }

  return {
    schemaVersion: config.schemaVersion,
    outputDisplayName: path.basename(config.output) || path.parse(config.output).root,
    options: structuredClone(config.options),
    fonts,
    tempFields: await Promise.all(config.tempFields.map(field => publicField(field, registry))),
    customTempFields: await Promise.all(config.customTempFields.map(field => publicField(field, registry))),
    temps: structuredClone(config.temps),
  }
}
