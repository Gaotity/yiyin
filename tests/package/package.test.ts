import { mkdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import { describe, expect, it } from 'vitest'

const root = process.cwd()

describe('release-age gate', () => {
  it('rejects releases younger than 24 hours and accepts older releases', async () => {
    const path = resolve(root, 'scripts/check-release-age.mjs')
    const module = await importIfPresent<{
      assertMinimumAge(
        name: string,
        version: string,
        publishedAt: string,
        now: number,
      ): void
    }>(path)
    expect(module).not.toBeNull()
    if (!module) {
      return
    }
    const now = Date.parse('2026-07-17T00:00:00Z')
    expect(() =>
      module.assertMinimumAge(
        'safe-package',
        '1.0.0',
        '2026-07-15T23:59:59Z',
        now,
      ),
    ).not.toThrow()
    expect(() =>
      module.assertMinimumAge(
        'new-package',
        '1.0.0',
        '2026-07-16T12:00:00Z',
        now,
      ),
    ).toThrow('new-package@1.0.0 is less than 24 hours old')
  })
})

describe('bundle verification', () => {
  it('accepts the pnpm script argument separator', async () => {
    const path = resolve(root, 'scripts/verify-bundle.mjs')
    const module = await importIfPresent<{
      parseArgs(argv: string[]): { platform: string; artifact: string }
    }>(path)
    expect(module).not.toBeNull()
    if (!module) {
      return
    }

    expect(
      module.parseArgs([
        '--',
        '--platform',
        'macos',
        '--artifact',
        'target/release/bundle/macos/壹印.app',
      ]),
    ).toEqual({
      platform: 'macos',
      artifact: 'target/release/bundle/macos/壹印.app',
    })
  })

  it('accepts an unsigned path-clean macOS app and rejects forbidden payloads', async () => {
    const path = resolve(root, 'scripts/verify-bundle.mjs')
    const module = await importIfPresent<{
      verifyBundle(options: {
        platform: 'macos' | 'windows'
        artifact: string
        root: string
      }): string[]
    }>(path)
    expect(module).not.toBeNull()
    if (!module) {
      return
    }
    const fixture = join(
      tmpdir(),
      `yiyin-package-${process.pid}-${Date.now()}`,
      '壹印.app',
    )
    mkdirSync(join(fixture, 'Contents', 'MacOS'), { recursive: true })
    mkdirSync(join(fixture, 'Contents', 'Resources'), { recursive: true })
    writeFileSync(
      join(fixture, 'Contents', 'Info.plist'),
      plist('壹印', 'io.github.gaotity.yiyin', '1.6.0'),
    )
    writeFileSync(join(fixture, 'Contents', 'MacOS', '壹印'), 'native-binary')
    writeFileSync(join(fixture, 'Contents', 'Resources', 'icon.icns'), 'icon')

    expect(
      module.verifyBundle({ platform: 'macos', artifact: fixture, root }),
    ).toEqual([])

    writeFileSync(
      join(fixture, 'Contents', 'Resources', 'electron-source.js.map'),
      'svelte sharp https://example.com',
    )
    expect(
      module.verifyBundle({ platform: 'macos', artifact: fixture, root }),
    ).toEqual(
      expect.arrayContaining([
        expect.stringContaining('source map'),
        expect.stringContaining('electron'),
        expect.stringContaining('remote URL'),
      ]),
    )
  })
})

describe('W3C smoke command', () => {
  it('accepts the pnpm script argument separator', async () => {
    const path = resolve(root, 'scripts/w3c-smoke.mjs')
    const module = await importIfPresent<{
      parseArgs(argv: string[]): { application: string; baseUrl: string }
    }>(path)
    expect(module).not.toBeNull()
    if (!module) {
      return
    }

    expect(
      module.parseArgs(['--', '--application', 'target/release/yiyin.exe']),
    ).toEqual({
      application: 'target/release/yiyin.exe',
      baseUrl: 'http://127.0.0.1:4444',
    })
  })
})

async function importIfPresent<T>(path: string): Promise<T | null> {
  try {
    return (await import(`${pathToFileURL(path).href}?t=${Date.now()}`)) as T
  } catch (error) {
    if (
      error instanceof Error &&
      (error.message.includes('Cannot find module') ||
        error.message.includes('ERR_MODULE_NOT_FOUND'))
    ) {
      return null
    }
    throw error
  }
}

function plist(name: string, identifier: string, version: string): string {
  return `<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist><dict>
<key>CFBundleName</key><string>${name}</string>
<key>CFBundleIdentifier</key><string>${identifier}</string>
<key>CFBundleShortVersionString</key><string>${version}</string>
</dict></plist>`
}
