import type { FontDescriptor } from '../platform/resources'

export interface FontSettings {
  bold: boolean
  italic: boolean
  size: number
  font: string
  caseType: 'lowcase' | 'upcase' | 'default'
  color: string
}

export interface FontSettingsOverride extends Partial<FontSettings> {
  use?: boolean
}

export interface Position {
  top: number | null
  bottom: number | null
  left: number | null
  right: number | null
}

export interface FieldInfoItem<T = string> {
  use?: boolean
  forceUse?: boolean
  show?: boolean
  key: string
  name: string
  value: T
  wImg: string
  bImg: string
  type: 'text' | 'img'
  font: FontSettingsOverride
}

export type FieldValue = string | number | boolean

export interface Template {
  key: string
  name: string
  temp: string
  use: boolean
  type: 'system' | 'custom'
  height?: number
  font: FontSettings
  position?: Position
  verticalAlign: 'center' | 'baseline'
}

export interface OutputOptions {
  iot: boolean
  landscape: boolean
  solid_bg: boolean
  solid_color: string
  bg_rate: { w: number, h: number }
  bg_rate_show: boolean
  origin_wh_output: boolean
  radius: number
  radius_show: boolean
  shadow: number
  shadow_show: boolean
  font: string
  main_img_w_rate: number
  text_margin: number
  quality: number
  mini_top_bottom_margin: number
  bg_blur: number
  preview_show: boolean
}

export interface PublicConfig {
  schemaVersion: number
  outputDisplayName: string
  options: OutputOptions
  fonts: FontDescriptor[]
  tempFields: FieldInfoItem<FieldValue>[]
  customTempFields: FieldInfoItem<FieldValue>[]
  temps: Template[]
}

export type ConfigMutation = Pick<PublicConfig, 'options' | 'tempFields' | 'customTempFields' | 'temps'>
