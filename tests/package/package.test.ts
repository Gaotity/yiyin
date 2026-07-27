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

    const runtime = join(fixture, 'Contents', 'Resources', 'runtime.js')
    writeFileSync(runtime, 'http://ipc.localhost')
    expect(
      module.verifyBundle({ platform: 'macos', artifact: fixture, root }),
    ).toEqual([])

    for (const remoteUrl of [
      'http://ipc.localhost.evil.example',
      'http://ipc.localhost:80',
      'http://user@ipc.localhost',
      'http://%',
    ]) {
      writeFileSync(runtime, remoteUrl)
      expect(
        module.verifyBundle({ platform: 'macos', artifact: fixture, root }),
      ).toEqual(expect.arrayContaining([expect.stringContaining('remote URL')]))
    }

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

  const writeLargeBundle = (name: string) => {
    const fixture = join(
      tmpdir(),
      `yiyin-package-large-${process.pid}-${Date.now()}`,
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
    const large = Buffer.alloc(6 * 1024 * 1024, 'a')
    large.write('electron', 5 * 1024 * 1024 + 100, 'utf8')
    large.write(
      'https://evil.example/payload.js',
      6 * 1024 * 1024 - 100,
      'utf8',
    )
    writeFileSync(join(fixture, 'Contents', 'Resources', name), large)
    return fixture
  }

  it('scans payloads and remote URLs in text files larger than 5MB', async () => {
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
    const fixture = writeLargeBundle('main.js')

    expect(
      module.verifyBundle({ platform: 'macos', artifact: fixture, root }),
    ).toEqual(
      expect.arrayContaining([
        expect.stringContaining('forbidden payload electron'),
        expect.stringContaining('remote URL'),
      ]),
    )
  })

  it('skips content scanning for large binary files', async () => {
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
    const fixture = writeLargeBundle('main.bin')

    // Compiled binaries legitimately embed web-platform data tables whose
    // bytes resemble payload names and URLs, so content checks only apply
    // to text-like files; name-based checks still cover every file.
    expect(
      module.verifyBundle({ platform: 'macos', artifact: fixture, root }),
    ).toEqual([])
  })
})

describe('W3C smoke command', () => {
  it('accepts the pnpm script argument separator', async () => {
    const path = resolve(root, 'scripts/w3c-smoke.mjs')
    const module = await importIfPresent<{
      parseArgs(argv: string[]): { debuggerAddress: string; baseUrl: string }
    }>(path)
    expect(module).not.toBeNull()
    if (!module) {
      return
    }

    expect(
      module.parseArgs(['--', '--debugger-address', '127.0.0.1:9222']),
    ).toEqual({
      debuggerAddress: '127.0.0.1:9222',
      baseUrl: 'http://127.0.0.1:4444',
    })
  })

  it('waits for driver readiness before creating exactly one session', async () => {
    const path = resolve(root, 'scripts/w3c-smoke.mjs')
    const module = await importIfPresent<{
      waitForDriver(
        command: (
          method: string,
          path: string,
          body?: unknown,
          timeout?: number,
        ) => Promise<unknown>,
        timeout?: number,
      ): Promise<void>
      createSession(
        command: (
          method: string,
          path: string,
          body?: unknown,
          timeout?: number,
        ) => Promise<unknown>,
        debuggerAddress: string,
      ): Promise<unknown>
    }>(path)
    expect(module).not.toBeNull()
    if (!module) {
      return
    }

    const calls: Array<{
      method: string
      path: string
      body: unknown
      timeout: number | undefined
    }> = []
    let statusAttempts = 0
    const command = async (
      method: string,
      requestPath: string,
      body?: unknown,
      timeout?: number,
    ) => {
      calls.push({ method, path: requestPath, body, timeout })
      if (requestPath === '/status' && statusAttempts++ === 0) {
        throw new Error('driver is starting')
      }
      if (requestPath === '/session') {
        return { value: { sessionId: 'session-1' } }
      }
      return { value: { ready: true } }
    }

    await module.waitForDriver(command, 1_000)
    const session = await module.createSession(command, '127.0.0.1:9222')

    expect(session).toEqual({ value: { sessionId: 'session-1' } })
    expect(calls.filter(({ path }) => path === '/status')).toHaveLength(2)
    expect(calls.filter(({ path }) => path === '/session')).toHaveLength(1)
    expect(calls.at(-1)).toMatchObject({
      method: 'POST',
      path: '/session',
      timeout: 120_000,
      body: {
        capabilities: {
          alwaysMatch: {
            'ms:edgeOptions': { debuggerAddress: '127.0.0.1:9222' },
          },
        },
      },
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
