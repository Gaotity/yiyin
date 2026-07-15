import { Buffer } from 'node:buffer'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import {
  assertBatchSize,
  assertContainedPath,
  assertImageDimensions,
  detectFileKind,
  validateFileSize,
} from '../../electron/resources/policy'

describe('resource policy', () => {
  it('detects supported image and font signatures', () => {
    expect(detectFileKind(Buffer.from([0xFF, 0xD8, 0xFF, 0xE0]))).toBe('jpeg')
    expect(detectFileKind(Buffer.from([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]))).toBe('png')
    expect(detectFileKind(Buffer.from('RIFF1234WEBP'))).toBe('webp')
    expect(detectFileKind(Buffer.from('OTTO'))).toBe('otf')
    expect(detectFileKind(Buffer.from([0x00, 0x01, 0x00, 0x00]))).toBe('ttf')
    expect(detectFileKind(Buffer.from('script'))).toBe('unknown')
  })

  it('enforces batch, size, and decompressed pixel limits', () => {
    expect(() => assertBatchSize(0)).toThrow()
    expect(() => assertBatchSize(101)).toThrow()
    expect(() => validateFileSize('image', 50 * 1024 * 1024)).not.toThrow()
    expect(() => validateFileSize('image', 50 * 1024 * 1024 + 1)).toThrow()
    expect(() => validateFileSize('font', 10 * 1024 * 1024 + 1)).toThrow()
    expect(() => assertImageDimensions(10_000, 10_000)).not.toThrow()
    expect(() => assertImageDimensions(20_000, 20_000)).toThrow()
    expect(() => assertImageDimensions(0, 100)).toThrow()
  })

  it('rejects paths outside an allowed root', () => {
    const root = path.resolve('/safe/root')
    expect(assertContainedPath(root, path.join(root, 'image.jpg'))).toBe(path.join(root, 'image.jpg'))
    expect(() => assertContainedPath(root, path.resolve('/safe/escape.jpg'))).toThrow('outside')
  })
})
