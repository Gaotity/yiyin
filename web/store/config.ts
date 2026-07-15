import type { PublicConfig } from '@common/models/config'
import { Message } from '@ggchivalrous/db-ui'
import { writable } from 'svelte/store'

let initialized = false
let loading = false

const initialConfig: PublicConfig = {
  schemaVersion: 1,
  outputDisplayName: '',
  options: {
    main_img_w_rate: 90,
    text_margin: 0.4,
    quality: 100,
    mini_top_bottom_margin: 0,
    iot: false,
    landscape: false,
    solid_bg: false,
    origin_wh_output: true,
    radius: 2.1,
    radius_show: true,
    shadow: 6,
    shadow_show: true,
    bg_rate_show: true,
    font: 'system-ui',
    bg_rate: { w: 0, h: 0 },
    bg_blur: 100,
    solid_color: '#fff',
    preview_show: false,
  },
  fonts: [],
  tempFields: [],
  customTempFields: [],
  temps: [],
}

export const config = writable<PublicConfig>(initialConfig)

export async function getConfig() {
  loading = true
  const result = await window.platform.config.get()
  if (result.ok) config.set(result.data)
  else Message.error(`配置加载失败：${result.error.message}`)
  loading = false
}

export async function resetConfig() {
  const result = await window.platform.config.reset()
  if (!result.ok) {
    Message.error(`重置失败：${result.error.message}`)
    return
  }
  config.set(result.data)
  Message.success({ message: '重置成功' })
}

config.subscribe(async (value) => {
  if (!initialized || loading) return
  const result = await window.platform.config.update({
    options: value.options,
    tempFields: value.tempFields,
    customTempFields: value.customTempFields,
    temps: value.temps,
  })
  if (!result.ok) Message.error(`配置持久化失败：${result.error.message}`)
})

void getConfig().finally(() => {
  initialized = true
})
