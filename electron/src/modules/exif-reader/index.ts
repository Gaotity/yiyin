import type { ExifData } from '@common/models/exif'
import fs from 'node:fs'
import { emptyExifData } from '@common/models/exif'
import ExifReader from 'exifreader'

interface ExifTagLike {
  description?: unknown
  value?: unknown
}

function tagValue(tags: Record<string, ExifTagLike>, key: string) {
  const tag = tags[key]
  const value = tag?.description ?? tag?.value
  if (Array.isArray(value)) return value.join(', ')
  return value === undefined || value === null ? '' : String(value)
}

function numericText(value: string) {
  return value.match(/-?\d+(?:\.\d+)?/)?.[0] ?? ''
}

export function normalizeExifTags(tags: Record<string, ExifTagLike>): ExifData {
  const exif = emptyExifData()
  exif.Make = tagValue(tags, 'Make')
  exif.Model = tagValue(tags, 'Model')
  exif.LensMake = tagValue(tags, 'LensMake')
  exif.LensModel = tagValue(tags, 'LensModel')
  exif.ExposureTime = tagValue(tags, 'ExposureTime')
  exif.FNumber = numericText(tagValue(tags, 'FNumber'))
  exif.ISO = numericText(tagValue(tags, 'ISOSpeedRatings') || tagValue(tags, 'ISO'))
  exif.FocalLength = numericText(tagValue(tags, 'FocalLength'))
  exif.FocalLengthIn35mmFormat = numericText(tagValue(tags, 'FocalLengthIn35mmFilm')) || exif.FocalLength
  exif.ExposureProgram = tagValue(tags, 'ExposureProgram')
  exif.DateTimeOriginal = tagValue(tags, 'DateTimeOriginal').replace(/^(\d{4}):(\d{2}):(\d{2})/, '$1-$2-$3')
  exif.ExposureCompensation = tagValue(tags, 'ExposureBiasValue')
  exif.MeteringMode = tagValue(tags, 'MeteringMode')
  exif.WhiteBalance = tagValue(tags, 'WhiteBalance')
  return exif
}

export class ExifReaderService {
  constructor(private readonly filePath: string) {}

  parse() {
    try {
      const tags = ExifReader.load(fs.readFileSync(this.filePath))
      const exif = normalizeExifTags(tags as Record<string, ExifTagLike>)
      return Object.values(exif).every(value => value === '') ? null : exif
    }
    catch {
      return null
    }
  }
}
