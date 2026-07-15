import { describe, expect, it } from 'vitest'
import { ipcSchemas } from '../../electron/ipc/schemas'

describe('iPC schemas', () => {
  it('accepts valid task and config payloads', () => {
    expect(ipcSchemas.taskId.parse({ taskId: 'a'.repeat(32) })).toEqual({ taskId: 'a'.repeat(32) })
    expect(ipcSchemas.outputOptions.parse({
      iot: false,
      landscape: false,
      solid_bg: false,
      solid_color: '#ffffff',
      bg_rate: { w: 4, h: 3 },
      bg_rate_show: false,
      origin_wh_output: false,
      radius: 2.1,
      radius_show: true,
      shadow: 6,
      shadow_show: true,
      font: '',
      main_img_w_rate: 90,
      text_margin: 0.4,
      quality: 100,
      mini_top_bottom_margin: 0,
      bg_blur: 50,
      preview_show: false,
    }).quality).toBe(100)
  })

  it('rejects malformed identifiers, colors, ranges, and unexpected keys', () => {
    expect(() => ipcSchemas.taskId.parse({ taskId: '../file' })).toThrow()
    expect(() => ipcSchemas.outputOptions.parse({ quality: 101 })).toThrow()
    expect(() => ipcSchemas.font.parse({ file: { path: '/tmp/font.ttf', name: 'font.ttf' }, name: '', extra: true })).toThrow()
  })
})
