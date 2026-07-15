import type { FieldInfoItem, FieldValue, FontSettings, FontSettingsOverride, OutputOptions, Position, Template } from '@common/models/config'

export interface IConfig {
  schemaVersion: number
  version: string
  dir: string
  output: string
  cacheDir: string
  staticDir: string
  font: {
    path: string
    dir: string
    map: Record<string, string>
  }
  options: OutputOptions
  tempFields: FieldInfoItem<FieldValue>[]
  customTempFields: FieldInfoItem<FieldValue>[]
  temps: Template[]
}

export type IFont = FontSettings
export type IFontParam = FontSettingsOverride
export type IPosition = Position
export type IFieldInfoItem<T = string> = FieldInfoItem<T>
