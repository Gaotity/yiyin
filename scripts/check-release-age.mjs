import { execFile } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { promisify } from 'node:util'

const execFileAsync = promisify(execFile)
const minimumAgeMs = 24 * 60 * 60 * 1000

export function assertMinimumAge(name, version, publishedAt, now = Date.now()) {
  const published = Date.parse(publishedAt)
  if (!Number.isFinite(published)) {
    throw new Error(`Missing publication time for ${name}@${version}`)
  }
  if (now - published < minimumAgeMs) {
    throw new Error(`${name}@${version} is less than 24 hours old`)
  }
}

export function directNpmPackages(manifest) {
  return Object.entries({
    ...manifest.dependencies,
    ...manifest.devDependencies,
  })
    .map(([name, version]) => {
      if (!/^\d+\.\d+\.\d+$/.test(version)) {
        throw new Error(`${name} must use an exact stable version`)
      }
      return { ecosystem: 'npm', name, version }
    })
    .sort(comparePackages)
}

export function directCargoPackages(metadata) {
  const packages = new Map()
  for (const workspacePackage of metadata.packages) {
    for (const dependency of workspacePackage.dependencies) {
      if (!dependency.source) {
        continue
      }
      const version = dependency.req.replace(/^=/, '')
      if (!/^\d+\.\d+\.\d+$/.test(version)) {
        throw new Error(
          `${dependency.name} must use an exact stable Cargo version`,
        )
      }
      packages.set(`${dependency.name}@${version}`, {
        ecosystem: 'cargo',
        name: dependency.name,
        version,
      })
    }
  }
  return [...packages.values()].sort(comparePackages)
}

async function loadPackages() {
  const root = fileURLToPath(new URL('..', import.meta.url))
  const manifest = JSON.parse(
    await readFile(join(root, 'package.json'), 'utf8'),
  )
  const { stdout } = await execFileAsync(
    'cargo',
    ['metadata', '--format-version', '1', '--locked', '--no-deps'],
    { cwd: root, maxBuffer: 10 * 1024 * 1024 },
  )
  return [
    ...directNpmPackages(manifest),
    ...directCargoPackages(JSON.parse(stdout)),
  ]
}

async function publicationTime(pkg) {
  const cacheKey = `${pkg.ecosystem}-${pkg.name}-${pkg.version}`.replace(
    /[^a-zA-Z0-9._-]/g,
    '_',
  )
  const cached = await readCache(cacheKey)
  if (cached) {
    return cached.publishedAt
  }

  let publishedAt
  if (pkg.ecosystem === 'npm') {
    const metadata = await fetchJson(
      `https://registry.npmjs.org/${encodeURIComponent(pkg.name)}`,
    )
    publishedAt = metadata.time?.[pkg.version]
  } else {
    const metadata = await fetchJson(
      `https://crates.io/api/v1/crates/${encodeURIComponent(pkg.name)}/${pkg.version}`,
    )
    publishedAt = metadata.version?.created_at
  }
  if (typeof publishedAt !== 'string') {
    throw new Error(`Registry metadata is missing ${pkg.name}@${pkg.version}`)
  }
  await writeCache(cacheKey, { publishedAt })
  return publishedAt
}

async function fetchJson(url) {
  let lastError
  for (let attempt = 1; attempt <= 3; attempt += 1) {
    try {
      const response = await fetch(url, {
        headers: { 'user-agent': 'yiyin-release-age-check/1.0' },
        signal: AbortSignal.timeout(30_000),
      })
      if (response.ok) {
        return response.json()
      }
      lastError = new Error(
        `Registry request failed with ${response.status}: ${url}`,
      )
      if (response.status < 500 && response.status !== 429) {
        throw lastError
      }
    } catch (error) {
      lastError = error
    }
    if (attempt < 3) {
      await new Promise((resolvePromise) =>
        setTimeout(resolvePromise, attempt * 500),
      )
    }
  }
  throw lastError ?? new Error(`Registry request failed: ${url}`)
}

function cacheDirectory() {
  if (process.env.CI !== 'true') {
    return null
  }
  return join(process.env.RUNNER_TEMP ?? tmpdir(), 'yiyin-release-age')
}

async function readCache(key) {
  const directory = cacheDirectory()
  if (!directory) {
    return null
  }
  try {
    return JSON.parse(await readFile(join(directory, `${key}.json`), 'utf8'))
  } catch {
    return null
  }
}

async function writeCache(key, value) {
  const directory = cacheDirectory()
  if (!directory) {
    return
  }
  await mkdir(directory, { recursive: true })
  await writeFile(join(directory, `${key}.json`), JSON.stringify(value))
}

function comparePackages(left, right) {
  return `${left.ecosystem}:${left.name}`.localeCompare(
    `${right.ecosystem}:${right.name}`,
  )
}

async function main() {
  const packages = await loadPackages()
  const now = Date.now()
  for (const pkg of packages) {
    const publishedAt = await publicationTime(pkg)
    assertMinimumAge(pkg.name, pkg.version, publishedAt, now)
    console.log(`${pkg.ecosystem}: ${pkg.name}@${pkg.version} (${publishedAt})`)
  }
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await main()
}
