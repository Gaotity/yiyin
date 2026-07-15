import type { ConfigMutation, FieldInfoItem, FieldValue, OutputOptions, PublicConfig, Template } from '../models/config'
import type { ExifData } from '../models/exif'
import type { FontDescriptor, ResourceDescriptor, TaskDescriptor } from './resources'
import type { PlatformResult } from './result'

export interface PlatformFile {
  readonly name: string
}

export interface TextImageData {
  data: string
  w?: number
  h?: number
}

export interface TextRenderRequest {
  taskId: string
  exif: ExifData
  bgHeight: number
  options: OutputOptions
  fields: FieldInfoItem<FieldValue>[]
  temps: Template[]
}

export interface ShadowMaterialImage {
  resourceUrl: string
  w: number
  h: number
  top: number
  left: number
}

export interface ShadowRenderRequest {
  taskId: string
  material: {
    bg: ShadowMaterialImage
    main: ShadowMaterialImage[]
  }
  options: OutputOptions
  rate: number
}

export interface TaskProgress {
  taskId: string
  progress: number
}

export interface TaskFailure {
  taskId: string
  message: string
}

export interface PlatformBridge {
  app: {
    minimize: () => Promise<PlatformResult<true>>
    close: () => Promise<PlatformResult<true>>
  }
  config: {
    get: () => Promise<PlatformResult<PublicConfig>>
    update: (mutation: ConfigMutation) => Promise<PlatformResult<PublicConfig>>
    reset: () => Promise<PlatformResult<PublicConfig>>
    chooseOutputDirectory: () => Promise<PlatformResult<PublicConfig | null>>
    openOutputDirectory: () => Promise<PlatformResult<true>>
  }
  files: {
    registerImages: (files: readonly PlatformFile[]) => Promise<PlatformResult<TaskDescriptor[]>>
    registerFont: (file: PlatformFile, name: string) => Promise<PlatformResult<FontDescriptor>>
    removeFont: (name: string) => Promise<PlatformResult<boolean>>
    registerOverlay: (file: PlatformFile, slot: string) => Promise<PlatformResult<ResourceDescriptor>>
  }
  tasks: {
    start: () => Promise<PlatformResult<true>>
    preview: (taskId: string) => Promise<PlatformResult<string>>
    readExif: (taskId: string) => Promise<PlatformResult<ExifData | null>>
    clear: () => Promise<PlatformResult<true>>
    completeTextRender: (taskId: string, images: TextImageData[]) => Promise<PlatformResult<true>>
    completeShadowRender: (taskId: string, dataUrl: string) => Promise<PlatformResult<true>>
  }
  events: {
    onProgress: (listener: (event: TaskProgress) => void) => () => void
    onFailure: (listener: (event: TaskFailure) => void) => () => void
    onTextRender: (listener: (event: TextRenderRequest) => void) => () => void
    onShadowRender: (listener: (event: ShadowRenderRequest) => void) => () => void
  }
}
