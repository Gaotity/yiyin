import {
  closeSync,
  existsSync,
  lstatSync,
  openSync,
  readdirSync,
  readFileSync,
  readSync,
  statSync,
} from 'node:fs'
import { basename, extname, join, resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const expected = {
  identifier: 'io.github.gaotity.yiyin',
  productName: '壹印',
  version: '1.6.0',
}
const forbiddenPayloads = ['electron', 'sharp', 'svelte', 'db-ui']
const forbiddenSidecars = ['node', 'node.exe', 'ffmpeg', 'exiftool']
const wholeFileScanLimit = 5 * 1024 * 1024
const scanChunkSize = 1024 * 1024
const maxPayloadLength = Math.max(
  ...forbiddenPayloads.map((payload) => payload.length),
)

export function verifyBundle({ platform, artifact, root = process.cwd() }) {
  const errors = []
  const configPath = join(root, 'src-tauri', 'tauri.conf.json')
  if (!existsSync(configPath)) {
    return [`missing Tauri config: ${configPath}`]
  }
  const config = JSON.parse(readFileSync(configPath, 'utf8'))
  verifyConfig(config, root, errors)
  if (!existsSync(artifact)) {
    return [...errors, `missing ${platform} artifact: ${artifact}`]
  }

  if (platform === 'macos') {
    verifyMacos(artifact, errors)
  } else if (platform === 'windows') {
    verifyWindows(artifact, config, errors)
  } else {
    errors.push(`unsupported platform: ${platform}`)
  }
  scanPayload(artifact, errors)
  return errors
}

function verifyConfig(config, root, errors) {
  for (const key of ['identifier', 'productName', 'version']) {
    if (config[key] !== expected[key]) {
      errors.push(`unexpected ${key}: ${String(config[key])}`)
    }
  }
  const windows = config.app?.windows ?? []
  if (windows.length !== 1 || windows[0]?.devtools !== false) {
    errors.push('production DevTools must be disabled for one window')
  }
  const serialized = JSON.stringify(config)
  for (const key of ['externalBin', 'sidecar', 'remoteDomainAccessScope']) {
    if (serialized.includes(key)) {
      errors.push(`forbidden generic capability: ${key}`)
    }
  }
  const permissionsPath = join(root, 'src-tauri', 'capabilities', 'main.json')
  const capability = JSON.parse(readFileSync(permissionsPath, 'utf8'))
  const expectedPermissions = [
    'core:event:allow-listen',
    'core:event:allow-unlisten',
    'core:window:allow-start-dragging',
  ]
  if (
    !Array.isArray(capability.permissions) ||
    JSON.stringify(capability.permissions) !==
      JSON.stringify(expectedPermissions)
  ) {
    errors.push('capabilities must expose only typed task event listening')
  }
  for (const icon of config.bundle?.icon ?? []) {
    if (!existsSync(resolve(root, 'src-tauri', icon))) {
      errors.push(`missing configured icon: ${icon}`)
    }
  }
}

function verifyMacos(artifact, errors) {
  if (basename(artifact) !== `${expected.productName}.app`) {
    errors.push(`unexpected macOS app name: ${basename(artifact)}`)
  }
  const contents = join(artifact, 'Contents')
  const plistPath = join(contents, 'Info.plist')
  if (!existsSync(plistPath)) {
    errors.push('missing macOS Info.plist')
  } else {
    const plist = readFileSync(plistPath, 'utf8')
    verifyPlistValue(plist, 'CFBundleName', expected.productName, errors)
    verifyPlistValue(plist, 'CFBundleIdentifier', expected.identifier, errors)
    verifyPlistValue(
      plist,
      'CFBundleShortVersionString',
      expected.version,
      errors,
    )
  }
  if (!hasFile(join(contents, 'MacOS'))) {
    errors.push('missing macOS application executable')
  }
  if (!hasExtension(join(contents, 'Resources'), '.icns')) {
    errors.push('missing bundled macOS icon')
  }
  if (existsSync(join(contents, '_CodeSignature'))) {
    errors.push('macOS artifact must remain unsigned')
  }
}

function verifyWindows(artifact, config, errors) {
  const files = walk(artifact)
  const installers = files.filter((path) =>
    ['.exe', '.msi'].includes(extname(path).toLowerCase()),
  )
  if (
    !installers.some(
      (path) =>
        basename(path).includes(expected.productName) &&
        basename(path).includes(expected.version),
    )
  ) {
    errors.push('missing versioned Windows installer')
  }
  const windows = config.bundle?.windows
  if (
    windows?.certificateThumbprint !== null ||
    (windows?.timestampUrl ?? '') !== ''
  ) {
    errors.push('Windows artifact must remain unsigned')
  }
}

function scanPayload(artifact, errors) {
  for (const path of walk(artifact)) {
    const lowerPath = path.toLowerCase()
    if (extname(lowerPath) === '.map') {
      errors.push(`source map found: ${path}`)
    }
    for (const sidecar of forbiddenSidecars) {
      if (basename(lowerPath) === sidecar) {
        errors.push(`forbidden sidecar found: ${sidecar}`)
      }
    }
    const found = new Set()
    if (statSync(path).size <= wholeFileScanLimit) {
      scanText(
        readFileSync(path, 'utf8').toLowerCase(),
        path,
        lowerPath,
        found,
        true,
      )
    } else if (isTextLike(lowerPath)) {
      scanLargeFile(path, lowerPath, found)
    }
    errors.push(...found)
  }
}

// Payload needles and URL patterns are plain-text matches; running them
// against a compiled binary false-positives on embedded web-platform data
// tables. Large files are therefore content-scanned only when they are
// text-like — name-based checks above still apply to every file.
const textLikeExtensions = new Set([
  '.cjs',
  '.css',
  '.htm',
  '.html',
  '.js',
  '.json',
  '.mjs',
  '.plist',
  '.svg',
  '.txt',
  '.webmanifest',
  '.xml',
])

function isTextLike(lowerPath) {
  return textLikeExtensions.has(extname(lowerPath))
}

// Large text files are scanned in chunks so an oversized bundled script or
// asset is never exempt from payload and remote-URL checks.
function scanLargeFile(path, lowerPath, found) {
  const descriptor = openSync(path, 'r')
  try {
    const buffer = Buffer.alloc(scanChunkSize)
    const decoder = new TextDecoder('utf-8')
    let carry = ''
    for (;;) {
      const bytesRead = readSync(descriptor, buffer, 0, buffer.length, null)
      if (bytesRead === 0) {
        if (carry) {
          scanText(carry, path, lowerPath, found, true)
        }
        return
      }
      const chunk = decoder.decode(buffer.subarray(0, bytesRead), {
        stream: true,
      })
      carry = scanText(
        (carry + chunk).toLowerCase(),
        path,
        lowerPath,
        found,
        false,
      )
    }
  } finally {
    closeSync(descriptor)
  }
}

// Returns the unterminated tail that must be rescanned with the next chunk.
function scanText(text, path, lowerPath, found, flush) {
  for (const payload of forbiddenPayloads) {
    if (text.includes(payload)) {
      found.add(`forbidden payload ${payload}: ${path}`)
    }
  }
  const urlPattern = /https?:\/\/[^\s'"<>]+/g
  for (;;) {
    const match = urlPattern.exec(text)
    if (match === null) {
      break
    }
    if (!flush && match.index + match[0].length === text.length) {
      return text.slice(match.index)
    }
    reportRemoteUrl(match[0], lowerPath, found)
  }
  return text.slice(-(maxPayloadLength - 1))
}

function reportRemoteUrl(url, lowerPath, found) {
  const isApplePlistDtd =
    basename(lowerPath) === 'info.plist' &&
    url === 'http://www.apple.com/dtds/propertylist-1.0.dtd'
  if (!isAllowedIpcUrl(url) && !isApplePlistDtd) {
    found.add(`remote URL found: ${url}`)
  }
}

function isAllowedIpcUrl(value) {
  try {
    const url = new URL(value)
    const authority = value.match(/^http:\/\/([^/?#]+)/)?.[1]
    return (
      url.protocol === 'http:' &&
      url.hostname === 'ipc.localhost' &&
      authority === 'ipc.localhost' &&
      url.port === '' &&
      url.username === '' &&
      url.password === ''
    )
  } catch {
    return false
  }
}

function verifyPlistValue(plist, key, value, errors) {
  const pattern = new RegExp(
    `<key>${escapeRegExp(key)}</key>\\s*<string>${escapeRegExp(value)}</string>`,
  )
  if (!pattern.test(plist)) {
    errors.push(`unexpected macOS ${key}`)
  }
}

function hasFile(directory) {
  return (
    existsSync(directory) &&
    readdirSync(directory).some((name) => {
      const path = join(directory, name)
      return lstatSync(path).isFile()
    })
  )
}

function hasExtension(directory, extension) {
  return (
    existsSync(directory) &&
    readdirSync(directory).some(
      (name) => extname(name).toLowerCase() === extension,
    )
  )
}

function walk(root) {
  if (!existsSync(root)) {
    return []
  }
  if (!lstatSync(root).isDirectory()) {
    return [root]
  }
  return readdirSync(root).flatMap((name) => walk(join(root, name)))
}

function escapeRegExp(value) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

export function parseArgs(argv) {
  const args = argv[0] === '--' ? argv.slice(1) : argv
  const options = {}
  for (let index = 0; index < args.length; index += 2) {
    const key = args[index]
    const value = args[index + 1]
    if (!key?.startsWith('--') || !value) {
      throw new Error('Expected --platform <value> --artifact <path>')
    }
    options[key.slice(2)] = value
  }
  if (!['macos', 'windows'].includes(options.platform) || !options.artifact) {
    throw new Error('Expected --platform <macos|windows> --artifact <path>')
  }
  return options
}

function main() {
  const options = parseArgs(process.argv.slice(2))
  const errors = verifyBundle({
    platform: options.platform,
    artifact: resolve(options.artifact),
  })
  if (errors.length) {
    throw new Error(errors.join('\n'))
  }
  console.log(`Verified ${options.platform} bundle: ${options.artifact}`)
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  main()
}
