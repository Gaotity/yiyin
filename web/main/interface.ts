import type { FieldInfoItem, PublicConfig } from '@common/models/config'
import type { ExifData } from '@common/models/exif'
import type { TaskDescriptor } from '@common/platform/resources'

export type IFileInfo = TaskDescriptor
export type IConfig = PublicConfig
export type IFieldInfoItem<T = string> = FieldInfoItem<T>

export type TInputEvent = Event & {
  currentTarget: EventTarget & HTMLInputElement
}

export interface ImgInfo extends TaskDescriptor {
  exif: ExifData | null | undefined
  faild: boolean
  faildMsg: string
  progress: number
  closeInterval: () => void
}
