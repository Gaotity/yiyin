import type { ShadowRenderRequest } from '@common/platform/bridge'

export type ImageToolOption = Omit<ShadowRenderRequest, 'taskId'>
export type Material = ShadowRenderRequest['material']
