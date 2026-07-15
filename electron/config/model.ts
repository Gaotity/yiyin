import type { IConfig, IFieldInfoItem } from '../src/interface'
import path from 'node:path'
import { defTemps, exifFields } from '@common/const'

function createField<T>(value: T, key = '', name = ''): IFieldInfoItem<T> {
  return {
    key,
    name,
    show: true,
    forceUse: false,
    use: false,
    value,
    type: 'text',
    bImg: '',
    wImg: '',
    font: {
      use: false,
      bold: false,
      italic: false,
      size: 0,
      font: '',
      caseType: 'default',
      color: '',
    },
  }
}

export function createDefaultConfig(userDataPath: string, picturesPath: string, tempPath: string, version = '1.6.0'): IConfig {
  return {
    schemaVersion: 1,
    version,
    dir: path.join(userDataPath, 'config.json'),
    output: path.join(picturesPath, 'watermark'),
    cacheDir: path.join(tempPath, 'yiyin'),
    staticDir: path.join(userDataPath, 'static'),
    font: {
      path: path.join(userDataPath, 'font.json'),
      dir: path.join(userDataPath, 'font'),
      map: {},
    },
    options: {
      iot: false,
      landscape: false,
      solid_bg: false,
      solid_color: '#fff',
      origin_wh_output: false,
      radius: 2.1,
      radius_show: true,
      shadow: 6,
      shadow_show: true,
      bg_rate_show: false,
      bg_rate: { w: 0, h: 0 },
      font: '',
      main_img_w_rate: 90,
      text_margin: 0.4,
      quality: 100,
      mini_top_bottom_margin: 0,
      bg_blur: 100,
      preview_show: false,
    },
    tempFields: exifFields.map(item => createField(item.value, item.key, item.name)),
    customTempFields: [],
    temps: structuredClone(defTemps),
  }
}
