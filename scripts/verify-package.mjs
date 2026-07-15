import fs from 'node:fs'
import path from 'node:path'
import process from 'node:process'
import { FuseState, FuseV1Options, getCurrentFuseWire } from '@electron/fuses'

const root = path.resolve(process.argv[2] ?? 'dist-electron')
const executable = process.argv[3] ? path.resolve(process.argv[3]) : null
const forbiddenNames = /^(?:exiftool|ffmpeg)(?:\.exe)?$|^(?:windows-exiftool|commond-exiftool)(?:\.|-)/i
const forbiddenText = /GITHUB_TOKEN|GH_TOKEN|ggchivalrous\/yiyin|uploads\.github\.com\/repos\/ggchivalrous/i

function files(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const item = path.join(directory, entry.name)
    return entry.isDirectory() ? files(item) : [item]
  })
}

if (!fs.existsSync(root)) throw new Error(`Package target does not exist: ${root}`)
for (const file of files(root)) {
  if (fs.lstatSync(file).isSymbolicLink()) continue
  if (forbiddenNames.test(path.basename(file))) throw new Error(`Forbidden executable or archive: ${file}`)
  const stat = fs.statSync(file)
  if (stat.size <= 2 * 1024 * 1024) {
    const content = fs.readFileSync(file, 'utf8')
    if (forbiddenText.test(content)) throw new Error(`Forbidden release reference or token marker: ${file}`)
  }
}

if (executable) {
  const fuses = await getCurrentFuseWire(executable)
  const expected = new Map([
    [FuseV1Options.RunAsNode, FuseState.DISABLE],
    [FuseV1Options.EnableCookieEncryption, FuseState.ENABLE],
    [FuseV1Options.EnableNodeOptionsEnvironmentVariable, FuseState.DISABLE],
    [FuseV1Options.EnableNodeCliInspectArguments, FuseState.DISABLE],
    [FuseV1Options.EnableEmbeddedAsarIntegrityValidation, FuseState.ENABLE],
    [FuseV1Options.OnlyLoadAppFromAsar, FuseState.ENABLE],
    [FuseV1Options.LoadBrowserProcessSpecificV8Snapshot, FuseState.DISABLE],
    [FuseV1Options.GrantFileProtocolExtraPrivileges, FuseState.DISABLE],
    [FuseV1Options.WasmTrapHandlers, FuseState.ENABLE],
  ])
  for (const [fuse, value] of expected) {
    if (fuses[fuse] !== value) throw new Error(`Unexpected Electron fuse ${fuse}: ${String(fuses[fuse])}`)
  }
}

console.log(`Verified package boundary: ${root}`)
