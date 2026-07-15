import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import path from 'node:path'

import sharp from 'sharp'

export type FixtureFormat = 'jpeg' | 'png' | 'webp'
export type ExifProfile = 'generic' | 'nikon' | 'sony'

export interface FieldOverride {
  key: string
  type?: 'text' | 'img'
  use?: boolean
  forceUse?: boolean
  value?: string
  bImg?: string
  wImg?: string
  font?: {
    use?: boolean
    bold?: boolean
    italic?: boolean
    size?: number
    font?: string
    caseType?: 'lowcase' | 'upcase' | 'default'
    color?: string
  }
}

export interface CustomTemplate {
  key: string
  name: string
  temp: string
  use: boolean
  type: 'custom'
  verticalAlign: 'center' | 'baseline'
  font: {
    size: number
    font: string
    bold: boolean
    italic: boolean
    color: string
    caseType: 'lowcase' | 'upcase' | 'default'
  }
}

export interface LegacyScenario {
  id: string
  source: string
  inputFile: string
  format: FixtureFormat
  exifProfile: ExifProfile
  orientation?: 6
  options: Record<string, unknown>
  templateKeys: string[]
  fieldOverrides?: FieldOverride[]
  customTemplates?: CustomTemplate[]
  perceptualThreshold: {
    minSsim: number
    maxChangedPixelRatio: number
  }
}

const threshold = { minSsim: 0.97, maxChangedPixelRatio: 0.08 }
const defaultTemplates = ['make-model', 'exif-params']

export const legacyScenarios: LegacyScenario[] = [
  {
    id: 'portrait-default',
    source: 'static/最终效果.png',
    inputFile: 'portrait-default.png',
    format: 'png',
    exifProfile: 'generic',
    options: {},
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'landscape-default',
    source: 'static/最终效果.jpg',
    inputFile: 'landscape-default.jpg',
    format: 'jpeg',
    exifProfile: 'generic',
    options: {},
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'webp-default',
    source: 'static/最终效果.jpg',
    inputFile: 'webp-default.webp',
    format: 'webp',
    exifProfile: 'generic',
    options: {},
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'exif-orientation-6',
    source: 'static/最终效果.jpg',
    inputFile: 'exif-orientation-6.jpg',
    format: 'jpeg',
    exifProfile: 'generic',
    orientation: 6,
    options: {},
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'explicit-ratio-3x2',
    source: 'static/最终效果.png',
    inputFile: 'explicit-ratio-3x2.png',
    format: 'png',
    exifProfile: 'generic',
    options: { bg_rate_show: true, bg_rate: { w: 3, h: 2 } },
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'portrait-to-landscape',
    source: 'static/最终效果-竖转横.jpeg',
    inputFile: 'portrait-to-landscape.jpg',
    format: 'jpeg',
    exifProfile: 'generic',
    options: { landscape: true },
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'solid-white-no-shadow',
    source: 'static/最终效果.jpg',
    inputFile: 'solid-white-no-shadow.jpg',
    format: 'jpeg',
    exifProfile: 'generic',
    options: { solid_bg: true, solid_color: '#fff', shadow_show: false, radius_show: false },
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'blurred-shadow-radius',
    source: 'static/最终效果.jpg',
    inputFile: 'blurred-shadow-radius.jpg',
    format: 'jpeg',
    exifProfile: 'generic',
    options: { bg_blur: 100, radius: 2.1, radius_show: true, shadow: 6, shadow_show: true },
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
  {
    id: 'built-in-equivalent-focal',
    source: 'static/最终效果.jpg',
    inputFile: 'built-in-equivalent-focal.jpg',
    format: 'jpeg',
    exifProfile: 'nikon',
    options: {},
    templateKeys: ['make-model', 'exif-params'],
    perceptualThreshold: threshold,
  },
  {
    id: 'built-in-original-focal',
    source: 'static/最终效果.jpg',
    inputFile: 'built-in-original-focal.jpg',
    format: 'jpeg',
    exifProfile: 'nikon',
    options: {},
    templateKeys: ['make-model', 'exif-params-1'],
    perceptualThreshold: threshold,
  },
  {
    id: 'logo-light',
    source: 'static/最终效果.jpg',
    inputFile: 'logo-light.jpg',
    format: 'jpeg',
    exifProfile: 'sony',
    options: { solid_bg: false },
    templateKeys: ['make-model'],
    perceptualThreshold: threshold,
  },
  {
    id: 'logo-dark',
    source: 'static/最终效果.jpg',
    inputFile: 'logo-dark.jpg',
    format: 'jpeg',
    exifProfile: 'sony',
    options: { solid_bg: true, solid_color: '#fff' },
    templateKeys: ['make-model'],
    perceptualThreshold: threshold,
  },
  {
    id: 'custom-text-forced',
    source: 'static/最终效果.png',
    inputFile: 'custom-text-forced.png',
    format: 'png',
    exifProfile: 'generic',
    options: {},
    templateKeys: ['fixture-signature'],
    fieldOverrides: [
      {
        key: 'PersonalSign',
        type: 'text',
        use: true,
        forceUse: true,
        value: 'YIYIN FIXTURE',
        font: { use: true, bold: true, italic: false, size: 2.4, font: '', caseType: 'upcase', color: '#ffffff' },
      },
    ],
    customTemplates: [
      {
        key: 'fixture-signature',
        name: 'Fixture signature',
        temp: '{PersonalSign}',
        use: true,
        type: 'custom',
        verticalAlign: 'center',
        font: { size: 2.2, font: '', bold: false, italic: false, color: '', caseType: 'default' },
      },
    ],
    perceptualThreshold: threshold,
  },
  {
    id: 'bundled-custom-font',
    source: 'static/最终效果.jpg',
    inputFile: 'bundled-custom-font.jpg',
    format: 'jpeg',
    exifProfile: 'generic',
    options: { font: '千图小兔' },
    templateKeys: defaultTemplates,
    perceptualThreshold: threshold,
  },
]

const exifProfiles: Record<ExifProfile, string[]> = {
  generic: ['-Make=ACME CORPORATION', '-Model=Camera One', '-LensMake=ACME Optics', '-LensModel=Prime 35'],
  nikon: ['-Make=NIKON CORPORATION', '-Model=NIKON Z 7_2', '-LensMake=NIKON', '-LensModel=NIKKOR Z 35mm f/1.8 S'],
  sony: ['-Make=SONY', '-Model=ILCE-7RM5', '-LensMake=SONY', '-LensModel=FE 35mm F1.4 GM'],
}

const commonExif = [
  '-FNumber=2.8',
  '-ISO=200',
  '-FocalLength=35',
  '-FocalLengthIn35mmFormat=52',
  '-ExposureTime=1/125',
  '-DateTimeOriginal=2026:01:02 03:04:05',
  '-ExposureCompensation=0.3',
  '-WhiteBalance=Auto',
  '-ExposureProgram=Aperture-priority AE',
  '-MeteringMode=Multi-segment',
  '-XResolution=300',
  '-YResolution=300',
  '-ResolutionUnit=inches',
]

export async function prepareLegacyInputs(root: string): Promise<void> {
  const inputDirectory = path.join(root, 'tests/fixtures/input')
  const exiftool = path.join(root, 'dist-electron/exiftool/exiftool')
  fs.mkdirSync(inputDirectory, { recursive: true })

  if (!fs.existsSync(exiftool)) {
    throw new Error(`Legacy ExifTool is missing: ${exiftool}`)
  }

  for (const scenario of legacyScenarios) {
    const target = path.join(inputDirectory, scenario.inputFile)
    const pipeline = sharp(path.join(root, scenario.source)).withMetadata({ density: 300 })

    if (scenario.format === 'jpeg') {
      await pipeline.jpeg({ quality: 96, chromaSubsampling: '4:4:4' }).toFile(target)
    }
    else if (scenario.format === 'png') {
      await pipeline.png({ compressionLevel: 9 }).toFile(target)
    }
    else {
      await pipeline.webp({ quality: 96, lossless: false }).toFile(target)
    }

    const orientation = scenario.orientation === 6 ? '-Orientation=Rotate 90 CW' : '-Orientation=Horizontal (normal)'
    execFileSync(exiftool, ['-overwrite_original', ...exifProfiles[scenario.exifProfile], ...commonExif, orientation, target], {
      stdio: 'pipe',
    })
  }

  fs.copyFileSync(
    path.join(root, 'web/assets/font/千图小兔体.ttf'),
    path.join(inputDirectory, '千图小兔体.ttf'),
  )
}
