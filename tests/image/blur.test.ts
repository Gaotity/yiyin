import { Buffer } from 'node:buffer'
import sharp from 'sharp'
import { describe, expect, it } from 'vitest'
import { createBlurredBackground } from '../../electron/image/blur'

describe('sharp background blur', () => {
  it('produces the requested dimensions without an external executable', async () => {
    const input = await sharp({
      create: { width: 20, height: 20, channels: 3, background: '#ff0000' },
    }).png().toBuffer()

    const output = await createBlurredBackground(input, 40, 30, 50)
    const metadata = await sharp(output).metadata()

    expect(metadata.width).toBe(40)
    expect(metadata.height).toBe(30)
    expect(metadata.format).toBe('jpeg')
  })

  it('clamps the blur percentage to a safe Sharp sigma', async () => {
    const input = await sharp({
      create: { width: 4, height: 4, channels: 3, background: '#000000' },
    }).png().toBuffer()

    await expect(createBlurredBackground(input, 4, 4, -1)).resolves.toBeInstanceOf(Buffer)
    await expect(createBlurredBackground(input, 4, 4, 500)).resolves.toBeInstanceOf(Buffer)
  })
})
