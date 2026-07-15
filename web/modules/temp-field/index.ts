import type { ITemp } from '@common/const/def-temps'
import type { FieldInfoItem, FieldValue } from '@common/models/config'
import { ExifFormat } from '@common/modules/exif-format'

interface GetFieldTempInfoOpt {
  bgHeight: number
  fields: FieldInfoItem<FieldValue>[]
}

/**
 * 获取模版参数信息配置
 * @param exifInfo - 读取到的相机信息
 */
export async function getFieldTempInfo(exifInfo: Record<string, any>, opt: GetFieldTempInfoOpt) {
  const tempFieldRecord: Record<string, FieldInfoItem<FieldValue>> = {}
  const fileds = opt.fields

  for (const filed of fileds) {
    tempFieldRecord[filed.key] = {
      ...filed,
      font: filed.font && {
        ...filed.font,
        size: filed.font.size ? Math.round(opt.bgHeight * (filed.font.size / 100)) : 0,
      },
    }
  }

  return fillTempFieldInfo(tempFieldRecord, exifInfo)
}

interface IGetTempsConfOpts {
  bgHeight: number
  color: string
  defFont: string
}

export function getTextTempList(temps: ITemp[], opts: IGetTempsConfOpts): ITemp[] {
  return temps.map(temp => ({
    ...temp,
    font: {
      ...temp.font,
      font: temp.font.font || opts.defFont,
      color: temp.font.color || opts.color,
      size: opts.bgHeight * (temp.font.size / 100),
    },
  })).filter(i => i.use)
}

/**
 * 模版 Field 对象信息填充
 * @param tempFieldConf
 * @param exifInfo - 规范化的相机信息对象
 */
function fillTempFieldInfo(
  tempFieldConf: Record<string, FieldInfoItem<FieldValue>>,
  exifInfo?: Record<string, any>,
) {
  const exif = new ExifFormat(exifInfo || {})
  const tempFieldInfo: Record<string, FieldInfoItem<FieldValue>> = {}

  for (const [field, info] of Object.entries(tempFieldConf)) {
    let output = tempFieldInfo[field]
    if (!output) {
      const formatter = (exif._ as unknown as Record<string, (() => FieldValue) | undefined>)[field]
      const normalized: FieldInfoItem<FieldValue> = {
        ...info,
        type: 'text',
        bImg: '',
        wImg: '',
        value: formatter?.call(exif._) || '',
      }
      output = structuredClone(normalized)
      tempFieldInfo[field] = output
    }

    // 强制使用则看该配置是否启动 || 非强制使用则看是否有原始相机信息
    if (info.use && (info.forceUse || !output.value)) {
      output.type = info.type || 'text'
      output.value = `${info.value || ''}`
      output.bImg = `${info.bImg || ''}`
      output.wImg = `${info.wImg || ''}`
      output.font = info.font || output.font
    }
  }

  return tempFieldInfo
}
