import { Buffer } from 'node:buffer'
import path from 'node:path'

export const MAX_IMAGE_FILE_BYTES = 50 * 1024 * 1024
export const MAX_FONT_FILE_BYTES = 10 * 1024 * 1024
export const MAX_IMAGE_BATCH = 100
export const MAX_IMAGE_PIXELS = 100_000_000

export type SupportedFileKind = 'jpeg' | 'png' | 'webp' | 'otf' | 'ttf' | 'unknown'

export function detectFileKind(header: Uint8Array): SupportedFileKind {
  const bytes = Buffer.from(header)
  if (bytes.length >= 3 && bytes[0] === 0xFF && bytes[1] === 0xD8 && bytes[2] === 0xFF) return 'jpeg'
  if (bytes.length >= 8 && bytes.subarray(0, 8).equals(Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]))) return 'png'
  if (bytes.length >= 12 && bytes.subarray(0, 4).toString('ascii') === 'RIFF' && bytes.subarray(8, 12).toString('ascii') === 'WEBP') return 'webp'
  if (bytes.length >= 4 && bytes.subarray(0, 4).toString('ascii') === 'OTTO') return 'otf'
  if (bytes.length >= 4 && bytes.subarray(0, 4).equals(Buffer.from([0x00, 0x01, 0x00, 0x00]))) return 'ttf'
  return 'unknown'
}

export function assertBatchSize(size: number) {
  if (!Number.isInteger(size) || size < 1 || size > MAX_IMAGE_BATCH) {
    throw new Error(`Image batches must contain between 1 and ${MAX_IMAGE_BATCH} files`)
  }
}

export function validateFileSize(type: 'image' | 'font', size: number) {
  const maximum = type === 'image' ? MAX_IMAGE_FILE_BYTES : MAX_FONT_FILE_BYTES
  if (!Number.isSafeInteger(size) || size < 1 || size > maximum) {
    throw new Error(`${type} file exceeds the allowed size`)
  }
}

export function assertImageDimensions(width: number | undefined, height: number | undefined) {
  if (!width || !height || !Number.isSafeInteger(width) || !Number.isSafeInteger(height) || width * height > MAX_IMAGE_PIXELS) {
    throw new Error('Image dimensions exceed the allowed pixel limit')
  }
}

export function assertContainedPath(root: string, candidatePath: string) {
  const normalizedRoot = path.resolve(root)
  const candidate = path.resolve(candidatePath)
  const relative = path.relative(normalizedRoot, candidate)
  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('Resource path resolves outside the allowed root')
  }
  return candidate
}
