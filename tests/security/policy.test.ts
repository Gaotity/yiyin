import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()
const packageJson = JSON.parse(read('package.json')) as {
  scripts: Record<string, string>
  dependencies: Record<string, string>
  devDependencies: Record<string, string>
}

function read(path: string): string {
  return readFileSync(join(root, path), 'utf8')
}

describe('dependency and desktop security policy', () => {
  it('uses only exact direct npm versions and excludes forbidden runtimes', () => {
    const dependencies = {
      ...packageJson.dependencies,
      ...packageJson.devDependencies,
    }
    expect(
      Object.entries(dependencies).filter(
        ([, version]) => !/^\d+\.\d+\.\d+$/.test(version),
      ),
    ).toEqual([])
    expect(Object.keys(dependencies)).not.toEqual(
      expect.arrayContaining([
        'electron',
        'sharp',
        'svelte',
        'webdriverio',
        'selenium-webdriver',
        'zustand',
        'tailwindcss',
        'zod',
        'axios',
      ]),
    )
    expect(packageJson.scripts['check:release-age']).toBe(
      'node scripts/check-release-age.mjs',
    )
    expect(packageJson.scripts['verify:bundle']).toBe(
      'node scripts/verify-bundle.mjs',
    )
  })

  it('locks Cargo registry sources without git or prerelease dependencies', () => {
    expect(existsSync(join(root, 'Cargo.lock'))).toBe(true)
    const manifests = [
      'Cargo.toml',
      'src-tauri/Cargo.toml',
      'crates/yiyin-domain/Cargo.toml',
      'crates/yiyin-application/Cargo.toml',
      'crates/yiyin-infrastructure/Cargo.toml',
    ].map(read)
    const lockfile = read('Cargo.lock')
    expect([...manifests, lockfile].join('\n')).not.toMatch(/git\s*=|git\+/)
    expect(lockfile).not.toMatch(/version = "\d+\.\d+\.\d+-[^"\s]+"/)
    expect(read('deny.toml')).toContain('unknown-git = "deny"')
  })

  it('keeps production DevTools, sidecars, and generic capabilities disabled', () => {
    const config = JSON.parse(read('src-tauri/tauri.conf.json')) as Record<
      string,
      unknown
    >
    const serialized = JSON.stringify(config)
    expect(config).toMatchObject({
      productName: '壹印',
      version: '1.6.0',
      identifier: 'io.github.gaotity.yiyin',
      app: { windows: [{ devtools: false }] },
    })
    expect(serialized).not.toMatch(
      /externalBin|sidecar|remoteDomainAccessScope/,
    )
    const csp = (config.app as { security: { csp: string } }).security.csp
    expect(csp.match(/https?:\/\/[^\s;]+/g) ?? []).toEqual([
      'http://ipc.localhost',
    ])

    const capability = JSON.parse(read('src-tauri/capabilities/main.json')) as {
      permissions: unknown[]
    }
    expect(capability.permissions).toEqual([
      'core:event:allow-listen',
      'core:event:allow-unlisten',
    ])
  })

  it('defines an opt-in fixture feature with no production default', () => {
    const manifest = read('src-tauri/Cargo.toml')
    const library = read('src-tauri/src/lib.rs')
    const app = read('src-tauri/src/app.rs')
    const fixture = read('src-tauri/src/e2e.rs')
    const packaging = read('.github/workflows/package.yml')
    expect(manifest).toMatch(/\[features\][\s\S]*default\s*=\s*\[\]/)
    expect(manifest).toMatch(/e2e-fixture\s*=\s*\[\]/)
    expect(library).toContain('#[cfg(feature = "e2e-fixture")]')
    expect(app).toContain('#[cfg(feature = "e2e-fixture")]')
    expect(fixture).toContain('YIYIN_E2E_FIXTURE_DIR')
    expect(`${library}\n${app}`).not.toContain('YIYIN_E2E_FIXTURE_DIR')
    expect(packaging).toContain(
      'pnpm tauri build --bundles nsis\n      - name: Verify production package contents',
    )
    expect(packaging).toContain(
      'cargo build -p yiyin-desktop --release --locked --features e2e-fixture',
    )
  })
})

describe('GitHub automation policy', () => {
  const requiredWorkflows = [
    '.github/workflows/ci.yml',
    '.github/workflows/codeql.yml',
    '.github/workflows/package.yml',
  ]

  it('pins every selected action in the new workflows to a full SHA', () => {
    for (const path of requiredWorkflows) {
      expect(existsSync(join(root, path)), path).toBe(true)
      if (!existsSync(join(root, path))) {
        continue
      }
      const uses = [...read(path).matchAll(/^\s*-?\s*uses:\s*([^\s#]+)/gm)].map(
        ([, value]) => value,
      )
      expect(uses.length, path).toBeGreaterThan(0)
      expect(
        uses.filter((value) => !/^[^@]+@[0-9a-f]{40}$/.test(value ?? '')),
      ).toEqual([])
    }
  })

  it('defines the required isolated quality, security, and package jobs', () => {
    if (!requiredWorkflows.every((path) => existsSync(join(root, path)))) {
      return
    }
    const ci = read(requiredWorkflows[0] ?? '')
    const codeql = read(requiredWorkflows[1] ?? '')
    const packaging = read(requiredWorkflows[2] ?? '')
    for (const job of [
      'frontend-quality',
      'rust-quality',
      'security',
      'golden-compatibility',
    ]) {
      expect(ci).toContain(`  ${job}:`)
    }
    expect(codeql).toContain('javascript-typescript')
    expect(codeql).toContain('rust')
    expect(packaging).toContain('  macos-package-smoke:')
    expect(packaging).toContain('  windows-package-and-desktop-smoke:')
    expect([ci, codeql, packaging].join('\n')).toContain('24.x')
    expect([ci, packaging].join('\n')).toContain('pnpm@11.13.0')
    expect([ci, codeql, packaging].join('\n')).toContain('1.97.0')
    expect(packaging).toContain('CARGO_DENY_VERSION:')
    expect(packaging).toContain('TAURI_DRIVER_VERSION:')
    expect(packaging).not.toContain('src-tauri/target/')
    expect(packaging).toContain('target/release/bundle/macos/壹印.app')
    expect(packaging).toContain('target/release/yiyin.exe')
  })

  it('separates Dependabot by ecosystem and contains no auto-merge', () => {
    const path = '.github/dependabot.yml'
    expect(existsSync(join(root, path))).toBe(true)
    if (!existsSync(join(root, path))) {
      return
    }
    const dependabot = read(path)
    expect(
      [...dependabot.matchAll(/package-ecosystem:\s*"([^"]+)"/g)].map(
        (match) => match[1],
      ),
    ).toEqual(['npm', 'cargo', 'github-actions'])
    expect(dependabot.toLowerCase()).not.toContain('auto-merge')
  })

  it('uses only the internal fetch-based W3C smoke client', () => {
    const path = 'scripts/w3c-smoke.mjs'
    expect(existsSync(join(root, path))).toBe(true)
    if (!existsSync(join(root, path))) {
      return
    }
    const source = read(path)
    expect(source).toContain('fetch(')
    expect(source).not.toMatch(/webdriverio|selenium/i)
  })
})
