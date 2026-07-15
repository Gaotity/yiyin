import { describe, expect, it } from 'vitest'
import { isAllowedExternalUrl, isTrustedRendererUrl } from '../../electron/security/urls'

describe('uRL security policy', () => {
  it('allows only explicit HTTPS hosts', () => {
    expect(isAllowedExternalUrl('https://github.com/Gaotity/yiyin')).toBe(true)
    expect(isAllowedExternalUrl('http://github.com/Gaotity/yiyin')).toBe(false)
    expect(isAllowedExternalUrl('https://github.com.evil.example/Gaotity/yiyin')).toBe(false)
    expect(isAllowedExternalUrl('not a url')).toBe(false)
  })

  it('accepts only the packaged app origin in production', () => {
    expect(isTrustedRendererUrl('yiyin://app/main/index.html', false)).toBe(true)
    expect(isTrustedRendererUrl('yiyin://resource/abc', false)).toBe(false)
    expect(isTrustedRendererUrl('file:///tmp/index.html', false)).toBe(false)
  })

  it('accepts only the loopback development origin', () => {
    expect(isTrustedRendererUrl('http://127.0.0.1:5173/main/index.html', true)).toBe(true)
    expect(isTrustedRendererUrl('http://localhost:5173/main/index.html', true)).toBe(false)
    expect(isTrustedRendererUrl('http://127.0.0.1:3000/main/index.html', true)).toBe(false)
    expect(isTrustedRendererUrl('not a url', true)).toBe(false)
  })
})
