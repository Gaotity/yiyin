import type { TextImageData } from '@common/platform/bridge'
import type { ImageTool } from '@modules/image-tool'
import { Queue } from '@modules/queue'

export const genTextImgQueue = new Queue<{
  id: string
  textImgList: TextImageData[]
}>({ concurrency: 2 })

export const genMainImgShadowQueue = new Queue<{
  id: string
  data: string
}>({ concurrency: 2 })

export const imageToolQueue = new Queue<ImageTool>({ concurrency: 2, autoRun: false })
