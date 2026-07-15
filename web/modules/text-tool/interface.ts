import type { ITemp } from '@common/const/def-temps'
import type { FieldInfoItem, FieldValue, OutputOptions } from '@common/models/config'
import type { ExifData } from '@common/models/exif'

export interface TextToolOption {
  options: OutputOptions
  temps: ITemp[]
  exif: ExifData
  bgHeight: number
  fields: FieldInfoItem<FieldValue>[]
}

export interface IFont {
  bold: boolean
  italic: boolean
  size: number
  font: string
  caseType: 'lowcase' | 'upcase' | 'default'
  color: string
}

export interface IFontParam extends Partial<IFont> {
  use?: boolean
}

export interface IImgFileInfo {
  data: string
  w?: number
  h?: number
}

export interface ISlotInfo {
  value: string | HTMLImageElement
  font: FieldInfoItem['font']
}

export interface ITextOption extends Pick<ITemp, 'height' | 'font' | 'verticalAlign'> {
  bgHeight: number
}

export interface TextInfo {
  color: string
  font: string
  value: string | HTMLImageElement
  type: 'text' | 'img'
  w: number
  x: number
  y: number
  h: number
}

export type TFontParam = Omit<IFontParam, 'offset'>
