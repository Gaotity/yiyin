import type { Exif } from '@modules/exif-reader/interface'
import type { IConfig } from '@src/interface'

import type { ImageToolOption, Material, OutputFilePaths, SizeInfo } from './interface'
import { Buffer } from 'node:buffer'
import Event from 'node:events'
import fs from 'node:fs'
import { join } from 'node:path'
import { ExifReaderService } from '@modules/exif-reader'
import { Logger } from '@modules/logger'
import { createBlurredBackground } from '@root/image/blur'
import { ipcChannels } from '@root/ipc/channels'
import { resourceRegistry } from '@root/resources'
import { mainApp } from '@src/common/app'
import { genMainImgShadowQueue, genTextImgQueue } from '@src/common/queue'
import { config } from '@src/config'
import { getFileName, md5, tryCatch, usePromise } from '@utils'

import sharp from 'sharp'

type SharpInstance = ReturnType<typeof sharp>
type SharpMetadata = Awaited<ReturnType<SharpInstance['metadata']>>
type SharpOverlays = Parameters<SharpInstance['composite']>[0]
type BackgroundColor = string | { r: number, g: number, b: number, alpha?: number }

const log = new Logger('ImageTool')
const NotInit = Symbol('未初始化')

interface EventMap {
  progress: (id: string, progress: number) => void
}

export class ImageTool extends Event {
  private isInit = false

  private isCancelled = false

  readonly id: string

  readonly path: string

  readonly name: string

  private outputOpt: IConfig['options']

  private outputFileNames: OutputFilePaths

  private meta!: SharpMetadata

  private sizeInfo!: SizeInfo

  private exif: Exif | null = null

  private _progress = 0

  private material: Material = {
    bg: { path: '', w: 0, h: 0, top: 0, left: 0 },
    main: [],
    text: [],
  }

  private contentH = 0

  // eslint-disable-next-line accessor-pairs
  set progress(n: number) {
    this._progress = n
    this.emit('progress', this.id, this._progress)
  }

  constructor(path: string, name: string, opt: ImageToolOption, taskId?: string) {
    super()

    this.path = path
    this.name = name
    this.outputOpt = opt.outputOption
    this.id = taskId ?? md5(`${md5(path)}${Math.random()}${Date.now()}`)

    const baseFilePath = join(opt.cachePath, this.id)
    this.outputFileNames = {
      base: baseFilePath,
      bg: `${baseFilePath}_bg.jpg`,
      main: `${baseFilePath}_main.jpg`,
      mask: `${baseFilePath}_mask.png`,
      composite: join(opt.outputPath, getFileName(opt.outputPath, name)),
    }
  }

  cancel() {
    this.isCancelled = true
    log.info('【%s】任务已取消', this.id)
  }

  async init() {
    if (this.isInit) return
    this.isInit = true

    // 准备基础信息
    const imgSharp = sharp(this.path).rotate()

    this.meta = await imgSharp.metadata()
    const { info: imgInfo } = await imgSharp.toBuffer({ resolveWithObject: true })
    this.sizeInfo = {
      w: imgInfo.width,
      h: imgInfo.height,
      resetW: imgInfo.width,
      resetH: imgInfo.height,
    }

    const { outputOpt } = this

    // 重置宽高比
    if (outputOpt.bg_rate_show && outputOpt.bg_rate.w && outputOpt.bg_rate.h) {
      const rate = +outputOpt.bg_rate.w / +outputOpt.bg_rate.h

      if (this.sizeInfo.w >= this.sizeInfo.h) {
        this.sizeInfo.resetH = Math.round(this.sizeInfo.w / rate)
      }
      else {
        this.sizeInfo.resetW = Math.round(this.sizeInfo.h * rate)
      }
    }

    // 横屏输出
    const width = outputOpt.landscape && this.sizeInfo.resetW < this.sizeInfo.resetH
      ? this.sizeInfo.resetH
      : this.sizeInfo.resetW
    const height = outputOpt.landscape && this.sizeInfo.resetW < this.sizeInfo.resetH
      ? this.sizeInfo.resetW
      : this.sizeInfo.resetH

    this.sizeInfo.resetW = width
    this.sizeInfo.resetH = height

    // 获取相机信息
    const exifReader = new ExifReaderService(this.path)
    this.exif = exifReader.parse()
  }

  async genWatermark() {
    this.progress = 1
    log.info('【%s】初始化基础数据', this.id)
    await this.init()
    this.progress = 10

    log.info('【%s】初步计算背景图片大小', this.id)
    this.clacBgImgSize()
    this.progress = 20

    log.info('【%s】生成文本图片', this.id)
    await this.genTextImg()
    this.progress = 30

    log.info('【%s】生成主图', this.id)
    await this.genMainImg()
    this.progress = 50

    log.info('【%s】计算内容高度', this.id)
    this.calcContentHeight()
    this.progress = 60

    log.info('【%s】生成背景图', this.id)
    await this.genBgImg()
    this.progress = 70

    log.info('【%s】生成主图阴影遮罩', this.id)
    await this.genMainImgShadow()
    this.progress = 90

    log.info('【%s】图片合成...', this.id)
    await this.composite()
    this.progress = 100

    this.delCacheFile()
  }

  async genPreview() {
    log.info('【%s】生成预览图...', this.id)
    if (this.isCancelled) return null
    await this.init()
    if (this.isCancelled) return null
    this.clacBgImgSize()
    if (this.isCancelled) return null
    await this.genTextImg()
    if (this.isCancelled) return null
    await this.genMainImg()
    if (this.isCancelled) return null
    this.calcContentHeight()
    if (this.isCancelled) return null
    await this.genBgImg()
    if (this.isCancelled) return null
    await this.genMainImgShadow()
    if (this.isCancelled) return null
    const res = await this.composite(true)
    this.delCacheFile()
    return res
  }

  async genBgImg() {
    const toFilePath: string = this.outputFileNames.bg
    this.clacBgImgSize(this.contentH)
    const { w, h } = this.material.bg

    if (this.outputOpt.solid_bg) {
      await this.genSolidImg(w, h, toFilePath)
    }
    else {
      await this.genBlurImg(w, h, toFilePath)
    }

    const mainImage = this.material.main[0]
    if (!mainImage) throw new Error('Main image material is missing')
    mainImage.left = Math.round((this.material.bg.w - mainImage.w) / 2)
    mainImage.top += Math.round((this.material.bg.h - this.contentH) / 2)
  }

  async genMainImg() {
    const toFilePath: string = this.outputFileNames.main
    if (!this.isInit) throw NotInit
    await sharp(this.path)
      .rotate()
      .withMetadata({ density: this.meta.density })
      .toFormat('jpeg', { quality: 100 })
      .toFile(toFilePath)

    this.material.main.push({
      path: toFilePath,
      w: this.sizeInfo.w,
      h: this.sizeInfo.h,
      top: 0,
      left: 0,
    })
  }

  async genTextImg() {
    if (this.isCancelled) return
    const [p, r, j] = usePromise()
    let timer: NodeJS.Timeout

    const handler: Parameters<typeof genTextImgQueue.on>[number] = async ({ id, textImgList = [] }) => {
      if (id === this.id) {
        if (this.isCancelled) {
          clearTimeout(timer)
          genTextImgQueue.off(handler)
          r(false)
          return
        }
        this.material.text = textImgList.map(i => ({
          path: '',
          buf: Buffer.from(i.data.split(',')[1] ?? '', 'base64'),
          w: i.w ?? 0,
          h: i.h ?? 0,
          top: 0,
          left: 0,
        }))

        if (import.meta.env.DEV) {
          tryCatch(() => {
            for (const { buf } of this.material.text) {
              if (buf) fs.writeFileSync(join(`${this.outputFileNames.base}_${Date.now() + Math.random()}.png`), buf)
            }
          }, null, e => log.error('文字图片写入异常', e))
        }

        clearTimeout(timer)
        genTextImgQueue.off(handler)
        r(true)
      }
    }

    timer = setTimeout(() => {
      log.error('【%s】水印文字图片生成超时', this.id)
      genTextImgQueue.off(handler)
      j(new Error('水印文字图片生成超时'))
    }, 20e3)

    genTextImgQueue.on(handler)

    const fields = await Promise.all([...config.tempFields, ...config.customTempFields].map(async field => ({
      ...field,
      bImg: await this.toResourceUrl(field.bImg),
      wImg: await this.toResourceUrl(field.wImg),
    })))

    mainApp.win.webContents.send(ipcChannels.events.textRender, {
      taskId: this.id,
      exif: this.exif || {},
      bgHeight: this.material.bg.h,
      options: config.options,
      fields,
      temps: config.temps,
    })

    return p
  }

  async composite(isPreview = false) {
    const composite: SharpOverlays = []

    // 主图
    for (const img of this.material.main) {
      composite.push({ input: img.path, top: img.top, left: img.left })
    }

    // 背景
    composite.push({ input: this.outputFileNames.mask, gravity: sharp.gravity.center })

    // 文字
    if (this.material.text?.length) {
      const textCompositeList: SharpOverlays = []
      for (let i = this.material.text.length - 1; i >= 0; i--) {
        const text = this.material.text[i]
        if (!text?.buf) continue
        const _composite: SharpOverlays[number] = {
          input: text.buf,
          left: Math.round((this.material.bg.w - text.w) / 2),
        }

        if (!textCompositeList.length) {
          _composite.top = Math.round(this.material.bg.h - text.h)
        }
        else {
          _composite.top = Math.round((textCompositeList.at(-1)?.top ?? 0) - text.h)
        }

        textCompositeList.push(_composite)
      }

      composite.push(...textCompositeList)
    }

    const output = sharp({
      create: {
        channels: 3,
        width: this.material.bg.w,
        height: this.material.bg.h,
        background: {
          r: 255,
          g: 255,
          b: 255,
        },
      },
    })
      .withMetadata({ density: this.meta.density })
      .composite(composite)
      .toFormat('jpeg', { quality: isPreview ? 70 : (this.outputOpt.quality || 100) })

    if (isPreview) {
      const buf = await output.toBuffer()
      return `data:image/jpeg;base64,${buf.toString('base64')}`
    }

    await output.toFile(this.outputFileNames.composite)

    log.info('【%s】图片合成完毕，输出到文件: ', this.id, this.outputFileNames.composite)
    return true
  }

  private async genBlurImg(width: number, height: number, toFilePath: string) {
    const buf = await createBlurredBackground(this.path, width, height, this.outputOpt.bg_blur)
    fs.writeFileSync(toFilePath, buf)
  }

  private async genSolidImg(width: number, height: number, toFilePath: string, color?: BackgroundColor) {
    return sharp({
      create: {
        channels: 3,
        width,
        height,
        background: (typeof color === 'string' ? color : this.outputOpt.solid_color) || '#fff',
      },
    })
      .toFormat('jpeg')
      .toFile(toFilePath)
  }

  private delCacheFile() {
    for (const k in this.outputFileNames) {
      if (k === 'composite') continue

      const _path = (this.outputFileNames as any)[k]
      if (fs.existsSync(_path)) {
        tryCatch(() => fs.rmSync(_path))
      }
    }
  }

  async genMainImgShadow() {
    if (this.isCancelled) return
    const [p, r, j] = usePromise()
    let timer: NodeJS.Timeout

    // 限制生成宽高，生成后再缩放回来
    let rate = 1
    if (this.material.bg.w > 10240) {
      rate = 10240 / this.material.bg.w
    }

    const handler: Parameters<typeof genMainImgShadowQueue.on>[number] = async ({ id, data }) => {
      if (id === this.id) {
        if (this.isCancelled) {
          clearTimeout(timer)
          genMainImgShadowQueue.off(handler)
          r(false)
          return
        }
        fs.writeFileSync(this.outputFileNames.mask, Buffer.from(data.split(',')[1] ?? '', 'base64'))

        if (rate !== 1) {
          await sharp(this.outputFileNames.mask)
            .resize({ width: this.material.bg.w, height: this.material.bg.h, fit: 'fill' })
            .toFormat('png')
            .toFile(`${this.outputFileNames.mask}catch.png`)
          fs.rmSync(this.outputFileNames.mask)
          fs.renameSync(`${this.outputFileNames.mask}catch.png`, this.outputFileNames.mask)
        }

        clearTimeout(timer)
        r(true)
        genMainImgShadowQueue.off(handler)
      }
    }

    timer = setTimeout(() => {
      log.error('【%s】图片阴影生成超时', this.id)
      genMainImgShadowQueue.off(handler)
      j(new Error('图片阴影生成超时'))
    }, 20e3)

    genMainImgShadowQueue.on(handler)
    const bg = await resourceRegistry.registerManaged(this.material.bg.path, 'background.jpg', 'image')
    const main = await Promise.all(this.material.main.map(async item => ({
      ...item,
      resourceUrl: (await resourceRegistry.registerManaged(item.path, 'image.jpg', 'image')).resourceUrl,
    })))
    mainApp.win.webContents.send(ipcChannels.events.shadowRender, {
      taskId: this.id,
      material: {
        bg: { ...this.material.bg, resourceUrl: bg.resourceUrl },
        main,
      },
      options: config.options,
      rate,
    })

    return p
  }

  private async toResourceUrl(filePath: string) {
    if (!filePath) return ''
    if (filePath.startsWith('yiyin://resource/')) return filePath
    if (!fs.existsSync(filePath)) return ''
    return (await resourceRegistry.registerManaged(filePath, filePath, 'image')).resourceUrl
  }

  /**
   * @param height - 指定内容高度，默认为创建时的输入的图片高度
   */
  clacBgImgSize(height: number = this.sizeInfo.h) {
    if (!this.isInit) throw NotInit

    let resetHeight = this.sizeInfo.resetH
    let resetWidth = this.sizeInfo.resetW

    const whRate = resetWidth / resetHeight

    // 按照重置后的宽高比算出适合内容高度的宽度
    if (height) {
      resetHeight = height
      resetWidth = Math.ceil(resetHeight * whRate)
    }
    else {
      // 主图高度比重置后的高度高，需要使用主图高度作为最终高度
      const validHeight = this.sizeInfo.h > resetHeight ? this.sizeInfo.h : resetHeight
      resetHeight = validHeight
      resetWidth = Math.ceil(resetHeight * whRate)
    }

    // 如果重置后，宽度太窄，则等比扩大宽高
    const mainImgWidthRate = (this.outputOpt.main_img_w_rate || 90) / 100
    if (this.sizeInfo.w / resetWidth > mainImgWidthRate) {
      resetWidth = Math.ceil(this.sizeInfo.w / mainImgWidthRate)
      resetHeight = Math.ceil(resetWidth / whRate)
    }

    this.material.bg = {
      path: this.outputFileNames.bg,
      h: resetHeight,
      w: resetWidth,
      top: 0,
      left: 0,
    }
  }

  calcContentHeight() {
    const opt = this.outputOpt
    const bgHeight = this.material.bg.h
    const mainImgTopOffset = bgHeight * (opt.mini_top_bottom_margin / 100)
    const textButtomOffset = bgHeight * 0.027

    // 主图上下间隔最小间隔
    let contentTop = Math.ceil(mainImgTopOffset)
    let mainImgOffset = contentTop * 2

    // 阴影宽度
    if (opt.shadow_show) {
      const mainImage = this.material.main[0]
      if (!mainImage) throw new Error('Main image material is missing')
      const shadowHeight = Math.ceil(mainImage.h * ((opt.shadow || 0) / 100))
      contentTop = Math.max(contentTop, Math.ceil(shadowHeight))
      mainImgOffset = contentTop * 2
    }

    // 有文字时文字与主图的间隔要小于主图对顶部的间隔，并且底部间隔使用文字对底部的间隔
    if (this.material.text.length) {
      mainImgOffset *= 3 / 4
      mainImgOffset += textButtomOffset
    }

    // 文本高度
    const textH = this.material.text.reduce((n, i) => {
      n += i.h
      return n
    }, 0)

    // 生成背景图片
    const mainImage = this.material.main[0]
    if (!mainImage) throw new Error('Main image material is missing')
    const contentH = Math.ceil(textH + mainImage.h + mainImgOffset)

    mainImage.top = contentTop
    this.contentH = contentH

    if (this.material.text?.length) {
      const lastText = this.material.text.at(-1)
      if (lastText) lastText.h += textButtomOffset
    }
  }

  override emit<U extends keyof EventMap>(
    event: U,
    ...args: Parameters<EventMap[U]>
  ): boolean {
    return super.emit(event, ...args)
  }

  override off<U extends keyof EventMap>(
    eventName: U,
    listener: EventMap[U],
  ): this {
    super.off(eventName, listener)
    return this
  }

  override on<U extends keyof EventMap>(
    event: U,
    listener: EventMap[U],
  ): this {
    super.on(event, listener)
    return this
  }

  override once<U extends keyof EventMap>(
    event: U,
    listener: EventMap[U],
  ): this {
    super.once(event, listener)
    return this
  }
}
