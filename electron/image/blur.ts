import type { Buffer } from 'node:buffer'
import sharp from 'sharp'

export async function createBlurredBackground(input: Buffer | string, width: number, height: number, blurPercent: number) {
  const normalized = Math.min(100, Math.max(0, blurPercent))
  let pipeline = sharp(input).rotate().resize({ width, height, fit: 'fill' })
  if (normalized > 0) pipeline = pipeline.blur(0.3 + normalized / 5)
  return pipeline.jpeg({ quality: 70 }).toBuffer()
}
