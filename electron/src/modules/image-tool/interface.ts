import type { OutputOptions } from '@common/models/config'
import type { Buffer } from 'node:buffer'

export interface ImageToolOption {
  outputOption: OutputOption
  cachePath: string
  outputPath: string
}

export type OutputOption = OutputOptions

export interface OutputFilePaths {
  /**
   * 基础路径
   */
  base: string

  /**
   * 背景图文件路径
   */
  bg: string

  /**
   * 主图文件路径
   */
  main: string

  /**
   * 最终合成图文件路径
   */
  composite: string

  /**
   * 遮罩层
   */
  mask: string
}

export interface SizeInfo {
  w: number
  h: number
  resetW: number
  resetH: number
}

export interface CalcContentHeightOption {
  bgHeight: number
  mainHeight: number
  textHeights: number[]
  options: OutputOptions
}

export interface GenMainImgShadowOption {
  // bgImgPath: string
  // mainImgPath: string
  offsetTop: number
  options: OutputOptions
}

export interface CompositeOption {
  textButtomOffset: number
  // bgImgPath: string
  mainImgList: {
    path: string
    top: number
    left: number
  }[]
  textList: {
    path: Buffer
    w: number
    h: number
  }[]
}

export interface Material {
  /**
   * 背景图片素材信息
   */
  bg: Img

  /**
   * 主图素材信息
   */
  main: Img[]

  /**
   * 文本素材信息
   */
  text: Img[]
}

export interface Img {
  path: string
  buf?: Buffer
  w: number
  h: number
  top: number
  left: number
}
