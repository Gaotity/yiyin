import { execFileSync } from 'node:child_process'
import { existsSync, readFileSync } from 'node:fs'
import { join } from 'node:path'
import { describe, expect, it } from 'vitest'

const root = process.cwd()

function trackedFiles(): string[] {
  return execFileSync('git', ['ls-files', '-z'], {
    cwd: root,
    encoding: 'utf8',
  })
    .split('\0')
    .filter((path) => path && existsSync(join(root, path)))
}

const allowedProvenance = (path: string) =>
  path.startsWith('docs/superpowers/') ||
  path.startsWith('tests/fixtures/legacy/') ||
  path === 'tests/security/final-tree.test.ts' ||
  path === 'tests/security/policy.test.ts' ||
  path === 'tests/package/package.test.ts' ||
  path === 'scripts/verify-bundle.mjs'

describe('final Tauri-only repository tree', () => {
  it('contains no legacy runtime files or binary payloads', () => {
    const forbiddenPaths = trackedFiles().filter((path) => {
      const lower = path.toLowerCase()
      return (
        /^(electron|web|common)\//.test(lower) ||
        /(^|\/)(electron|svelte|ffmpeg|exiftool)(\/|[._-])/.test(lower) ||
        /(^|\/)build-(mac|win)-x64(-dev)?\.yml$/.test(lower) ||
        [
          'auto-release.config.ts',
          'dev.js',
          'eslint.config.ts',
          'release.js',
          'svelte.config.ts',
          'tests/legacy/capture.spec.ts',
          'tests/legacy/scenarios.ts',
        ].includes(lower)
      )
    })

    expect(forbiddenPaths).toEqual([])
  })

  it('contains no forbidden dependency or renderer bridge references', () => {
    const contentPatterns = [
      /@ggchivalrous\/db-ui/i,
      /\b(electron|svelte|sharp|ffmpeg|exiftool)\b/i,
      /\brouterConfig\b/,
      /on:genTextImg/,
      /on:genMainImgShadow/,
    ]
    const contentFiles = trackedFiles().filter(
      (path) =>
        !allowedProvenance(path) &&
        (/^(src|src-tauri|crates|scripts)\//.test(path) ||
          path === 'package.json' ||
          path === 'pnpm-lock.yaml'),
    )
    const hits = contentFiles.flatMap((path) => {
      const contents = readFileSync(join(root, path), 'utf8')
      return contentPatterns
        .filter((pattern) => pattern.test(contents))
        .map((pattern) => `${path}: ${pattern.source}`)
    })

    expect(hits).toEqual([])
  })
})
