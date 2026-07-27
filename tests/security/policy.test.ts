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
      'cargo build -p yiyin-desktop --release --locked --features e2e-fixture,tauri/custom-protocol',
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

  it('cancels superseded runs for the same pull request', () => {
    for (const path of requiredWorkflows) {
      const workflow = read(path)
      expect(workflow, path).toContain(
        `group: \${{ github.workflow }}-\${{ github.event.pull_request.number || github.run_id }}`,
      )
      expect(workflow, path).toContain('cancel-in-progress: true')
    }
  })

  it('defines the required isolated quality, security, and package jobs', () => {
    if (!requiredWorkflows.every((path) => existsSync(join(root, path)))) {
      return
    }
    const ci = read(requiredWorkflows[0] ?? '')
    const codeql = read(requiredWorkflows[1] ?? '')
    const packaging = read(requiredWorkflows[2] ?? '')
    for (const job of ['frontend-quality', 'rust-quality', 'security']) {
      expect(ci).toContain(`  ${job}:`)
    }
    expect(ci).not.toContain('  golden-compatibility:')
    expect(ci).not.toContain('runs-on: macos-')
    expect(ci).toContain('cargo test --workspace --locked')
    expect(ci).toContain('name: Upload golden rendering diffs')
    expect(ci).toMatch(
      /name: Upload golden rendering diffs[\s\S]*retention-days: 3/,
    )
    expect(codeql).toContain('javascript-typescript')
    expect(codeql).toContain('rust')
    expect(packaging).toContain('  macos-package-smoke:')
    expect(packaging).toContain('  windows-package-and-desktop-smoke:')
    expect([ci, codeql, packaging].join('\n')).toContain('24.x')
    expect([ci, packaging].join('\n')).toContain('pnpm@11.13.0')
    expect([ci, codeql, packaging].join('\n')).toContain('1.97.0')
    expect(packaging).toContain('CARGO_DENY_VERSION:')
    expect(packaging).toContain('TAURI_CONFIG')
    expect(packaging).not.toContain('src-tauri/target/')
    expect(packaging).toContain('target/release/bundle/macos/壹印.app')
    expect(packaging).toContain('target/release/yiyin.exe')
    for (const dependency of [
      'libwebkit2gtk-4.1-dev',
      'libayatana-appindicator3-dev',
      'librsvg2-dev',
      'libxdo-dev',
      'libssl-dev',
      'build-essential',
    ]) {
      expect(ci).toContain(dependency)
    }
    expect(packaging).toContain('msedgedriver.exe --port=4444')
    expect(packaging).toContain('--remote-debugging-port=9222')
    expect(packaging).toContain('Stop-Process -Name msedgedriver, yiyin')
    expect(packaging).toContain('name: Upload Windows smoke diagnostics')
  })

  it('runs native package checks only for relevant pull request changes', () => {
    const packaging = read('.github/workflows/package.yml')
    expect(packaging).toContain('  native-changes:')
    expect(packaging).toContain('runs-on: ubuntu-latest')
    expect(packaging).toContain('fetch-depth: 0')
    expect(packaging).toContain(
      'git diff --name-only -z "$BASE_SHA...$HEAD_SHA" > "$changed_files"',
    )
    expect(packaging).toContain('done < "$changed_files"')
    expect(packaging).not.toContain('done < <(git diff')
    expect(packaging).toContain('*.md|docs/*)')
    for (const pattern of [
      'src/*',
      'src-tauri/*',
      'crates/*',
      'assets/*',
      'icon/*',
      'tests/fixtures/*',
      'package.json',
      'pnpm-lock.yaml',
      'pnpm-workspace.yaml',
      'Cargo.toml',
      'Cargo.lock',
      'rust-toolchain.toml',
      'index.html',
      'vite.config.mjs',
      'tsconfig.json',
      'scripts/verify-bundle.mjs',
      'scripts/w3c-smoke.mjs',
      '.github/workflows/package.yml',
    ]) {
      expect(packaging).toContain(pattern)
    }
    expect(packaging.match(/needs: native-changes/g)?.length).toBe(2)
    expect(
      packaging.match(
        /if: \$\{\{ !cancelled\(\) && \(needs\['native-changes'\]\.result != 'success' \|\| needs\['native-changes'\]\.outputs\.should_run == 'true'\) \}\}/g,
      )?.length,
    ).toBe(2)
  })

  it('uploads only short-lived manual packages and failure diagnostics', () => {
    const packaging = read('.github/workflows/package.yml')
    expect(packaging).toMatch(
      /name: Upload unsigned macOS packages[\s\S]*?if: github\.event_name == 'workflow_dispatch'[\s\S]*?retention-days: 7/,
    )
    expect(packaging).toMatch(
      /name: Upload unsigned Windows package[\s\S]*?if: always\(\) && github\.event_name == 'workflow_dispatch'[\s\S]*?retention-days: 7/,
    )
    expect(packaging).toMatch(
      /name: Upload Windows smoke diagnostics[\s\S]*?if: failure\(\)[\s\S]*?retention-days: 3/,
    )
  })

  it('disables setup-node automatic package-manager caching before Corepack activation', () => {
    for (const path of [
      '.github/workflows/ci.yml',
      '.github/workflows/package.yml',
    ]) {
      const workflow = read(path)
      const setupNodeCount =
        workflow.match(/actions\/setup-node@/g)?.length ?? 0
      const disabledCacheCount =
        workflow.match(/package-manager-cache:\s*false/g)?.length ?? 0
      expect(disabledCacheCount, path).toBe(setupNodeCount)
    }
  })

  it('configures Renovate per ecosystem and contains no auto-merge', () => {
    expect(existsSync(join(root, '.github/dependabot.yml'))).toBe(false)
    const path = 'renovate.json'
    expect(existsSync(join(root, path))).toBe(true)
    if (!existsSync(join(root, path))) {
      return
    }
    const renovate = JSON.parse(read(path)) as {
      extends?: string[]
      labels?: string[]
      packageRules?: {
        matchManagers?: string[]
        matchUpdateTypes?: string[]
        groupName?: string
      }[]
    }
    expect(renovate.extends).toContain('config:recommended')
    expect(renovate.labels).toContain('dependencies')
    const rules = renovate.packageRules ?? []
    expect(rules.flatMap((rule) => rule.matchManagers ?? [])).toEqual([
      'npm',
      'cargo',
      'github-actions',
      'npm',
      'cargo',
      'github-actions',
    ])
    const groups = rules.filter((rule) => rule.groupName)
    expect(groups).toHaveLength(3)
    for (const group of groups) {
      expect(group.matchUpdateTypes).toEqual(['minor', 'patch'])
    }
    expect(read(path).toLowerCase()).not.toContain('automerge')
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
