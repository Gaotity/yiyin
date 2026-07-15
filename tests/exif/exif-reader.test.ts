import fs from 'node:fs'
import path from 'node:path'
import sharp from 'sharp'
import { describe, expect, it } from 'vitest'
import { ExifReaderService, normalizeExifTags } from '../../electron/src/modules/exif-reader/index'

describe('exifReader normalization', () => {
  it('maps expanded tag descriptions to stable EXIF fields', () => {
    const exif = normalizeExifTags({
      Make: { description: 'NIKON CORPORATION' },
      Model: { description: 'NIKON Z 8' },
      FNumber: { description: '2.8' },
      ISO: { value: 100 },
      FocalLength: { description: '50 mm' },
      ExposureTime: { description: '1/125' },
      DateTimeOriginal: { description: '2026:01:02 03:04:05' },
    })

    expect(exif).toMatchObject({
      Make: 'NIKON CORPORATION',
      Model: 'NIKON Z 8',
      FNumber: '2.8',
      ISO: '100',
      FocalLength: '50',
      ExposureTime: '1/125',
      DateTimeOriginal: '2026-01-02 03:04:05',
    })
  })

  it('returns empty stable fields for missing tags', () => {
    expect(normalizeExifTags({})).toEqual(expect.objectContaining({ Make: '', Model: '', ISO: '' }))
  })

  it('reads files in-process and safely handles invalid files', async () => {
    const filePath = path.join('/tmp', `yiyin-exif-${process.pid}.png`)
    fs.writeFileSync(filePath, await sharp({
      create: { width: 2, height: 2, channels: 3, background: '#ffffff' },
    }).png().toBuffer())
    try {
      expect(new ExifReaderService(filePath).parse()).toBeNull()
      expect(new ExifReaderService(`${filePath}.missing`).parse()).toBeNull()
    }
    finally {
      fs.rmSync(filePath)
    }
  })
})
