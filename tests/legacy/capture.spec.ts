import { createHash } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import fs from 'node:fs'
import os from 'node:os'
import path from 'node:path'
import { createRequire } from 'node:module'

import type { Page } from '@playwright/test'
import { _electron as electron, expect, test } from '@playwright/test'
import sharp from 'sharp'

import { legacyScenarios, prepareLegacyInputs } from './scenarios'

const root = path.resolve(import.meta.dirname, '../..')
const electronExecutable = createRequire(import.meta.url)('electron') as string
const committedOutputDirectory = path.join(root, 'tests/fixtures/legacy/output')
const committedMetadataDirectory = path.join(root, 'tests/fixtures/legacy/metadata')
const committedManifestPath = path.join(root, 'tests/fixtures/legacy/manifest.json')

interface CapturedGeometry {
  canvas: { width: number, height: number }
  mainRect: { x: number, y: number, width: number, height: number }
  textRows: { x: number, y: number, width: number, height: number }[]
  maskSurface: { width: number, height: number, scale: number }
  shadowBlur: number
  cornerRadius: number
}

interface CapturedScenario {
  id: string
  input: string
  inputSha256: string
  options: Record<string, unknown>
  templateKeys: string[]
  expectedOutput: string
  expectedMetadata: string
  output: { width: number, height: number, density: number | null }
  exactGeometry: CapturedGeometry
  perceptualThreshold: { minSsim: number, maxChangedPixelRatio: number }
}

interface CaptureResult {
  manifest: {
    schemaVersion: 1
    renderer: { name: 'electron-sharp-canvas', revision: string, sharp: string }
    scenarios: CapturedScenario[]
  }
  metadata: Record<string, unknown>
}

test('captures every legacy scenario repeatably', async () => {
  test.setTimeout(15 * 60 * 1000)
  await prepareLegacyInputs(root)

  resetDirectory(committedOutputDirectory)
  resetDirectory(committedMetadataDirectory)

  const first = await captureRun(committedOutputDirectory)
  writeCapture(first, committedMetadataDirectory, committedManifestPath)

  const repeatRoot = fs.mkdtempSync(path.join(os.tmpdir(), 'yiyin-legacy-repeat-'))
  const repeatOutput = path.join(repeatRoot, 'output')
  const repeatMetadata = path.join(repeatRoot, 'metadata')
  const repeatManifest = path.join(repeatRoot, 'manifest.json')

  try {
    const second = await captureRun(repeatOutput)
    writeCapture(second, repeatMetadata, repeatManifest)

    expect(second.manifest).toEqual(first.manifest)
    expect(sha256File(repeatManifest)).toBe(sha256File(committedManifestPath))

    for (const scenario of legacyScenarios) {
      const metadataName = `${scenario.id}.json`
      expect(sha256File(path.join(repeatMetadata, metadataName))).toBe(
        sha256File(path.join(committedMetadataDirectory, metadataName)),
      )

      const firstImage = await sharp(path.join(committedOutputDirectory, `${scenario.id}.jpg`)).metadata()
      const secondImage = await sharp(path.join(repeatOutput, `${scenario.id}.jpg`)).metadata()
      expect({ width: secondImage.width, height: secondImage.height, density: secondImage.density ?? null }).toEqual({
        width: firstImage.width,
        height: firstImage.height,
        density: firstImage.density ?? null,
      })
    }
  }
  finally {
    fs.rmSync(repeatRoot, { recursive: true, force: true })
  }
})

async function captureRun(outputDirectory: string): Promise<CaptureResult> {
  resetDirectory(outputDirectory)
  const fixtureHome = fs.mkdtempSync(path.join(os.tmpdir(), 'yiyin-legacy-home-'))
  const app = await electron.launch({
    executablePath: electronExecutable,
    args: [path.join(root, 'dist-electron')],
    cwd: root,
    env: {
      ...process.env,
      HOME: fixtureHome,
      YIYIN_FIXTURE_MODE: '1',
      YIYIN_FIXTURE_OUTPUT: outputDirectory,
    },
  })

  try {
    const page = await app.firstWindow()
    await page.waitForLoadState('domcontentloaded')
    await installCaptureListeners(page)

    const captured: CapturedScenario[] = []
    const metadata: Record<string, unknown> = {}

    for (const scenario of legacyScenarios) {
      const inputPath = path.join(root, 'tests/fixtures/input', scenario.inputFile)
      const browserResult = await page.evaluate(async ({ inputPath, scenario }) => {
        const fixtureWindow = window as typeof window & {
          api: Record<string, (data?: unknown) => Promise<any> | void>
          __yiyinFixtureEvents: {
            progress: { id: string, progress: number }[]
            failed: { id: string, msg: string }[]
            geometry: any[]
          }
          __yiyinFixtureBaseline?: any
        }

        if (!fixtureWindow.__yiyinFixtureBaseline) {
          const baseline = await fixtureWindow.api.getConfig()
          if (baseline.code !== 0) {
            throw new Error(baseline.message)
          }
          fixtureWindow.__yiyinFixtureBaseline = structuredClone(baseline.data)
        }
        const current = { code: 0, data: structuredClone(fixtureWindow.__yiyinFixtureBaseline) }

        const overrideRecord = Object.fromEntries((scenario.fieldOverrides ?? []).map(item => [item.key, item]))
        const tempFields = current.data.tempFields.map((field: any) => {
          const override = overrideRecord[field.key]
          if (!override) return field
          return { ...field, ...override, font: { ...field.font, ...override.font } }
        })
        const customTemplates = scenario.customTemplates ?? []
        const temps = [...current.data.temps.filter((item: any) => item.type === 'system'), ...customTemplates]
          .map((item: any) => ({ ...item, use: scenario.templateKeys.includes(item.key) }))

        const updated = await fixtureWindow.api.setConfig({
          ...current.data,
          options: { ...current.data.options, ...scenario.options },
          tempFields,
          customTempFields: [],
          temps,
        })
        if (updated.code !== 0) {
          throw new Error(updated.message)
        }

        const added = await fixtureWindow.api.addTask([{ path: inputPath, name: scenario.inputFile }])
        if (added.code !== 0 || !added.data?.[0]) {
          throw new Error(added.message || `Unable to register ${scenario.id}`)
        }

        const taskId = added.data[0].id
        const started = await fixtureWindow.api.startTask()
        if (started.code !== 0) {
          throw new Error(started.message)
        }

        const terminal = await new Promise<'completed'>((resolve, reject) => {
          const timeout = window.setTimeout(() => reject(new Error(`Timed out rendering ${scenario.id}`)), 120_000)
          const timer = window.setInterval(() => {
            const failed = fixtureWindow.__yiyinFixtureEvents.failed.find(event => event.id === taskId)
            if (failed) {
              window.clearInterval(timer)
              window.clearTimeout(timeout)
              reject(new Error(failed.msg))
              return
            }

            const completed = fixtureWindow.__yiyinFixtureEvents.progress.some(
              event => event.id === taskId && event.progress === 100,
            )
            if (completed) {
              window.clearInterval(timer)
              window.clearTimeout(timeout)
              resolve('completed')
            }
          }, 50)
        })

        const shadow = fixtureWindow.__yiyinFixtureEvents.geometry.find(event => event.id === taskId)
        if (!shadow) {
          throw new Error(`Missing geometry for ${scenario.id}`)
        }

        const exif = await fixtureWindow.api.getExitInfo(inputPath)
        if (exif.code !== 0) {
          throw new Error(exif.message)
        }

        const { material, options, rate } = shadow
        const textRows: { x: number, y: number, width: number, height: number }[] = []
        for (let index = material.text.length - 1; index >= 0; index -= 1) {
          const row = material.text[index]
          const previous = textRows.at(-1)
          textRows.push({
            x: Math.round((material.bg.w - row.w) / 2),
            y: previous ? Math.round(previous.y - row.h) : Math.round(material.bg.h - row.h),
            width: row.w,
            height: row.h,
          })
        }

        const scaledMainHeight = Math.ceil(material.main[0].h * rate)
        return {
          terminal,
          taskId,
          exif: exif.data ?? null,
          geometry: {
            canvas: { width: material.bg.w, height: material.bg.h },
            mainRect: {
              x: material.main[0].left,
              y: material.main[0].top,
              width: material.main[0].w,
              height: material.main[0].h,
            },
            textRows: textRows.reverse(),
            maskSurface: {
              width: Math.floor(material.bg.w * rate),
              height: Math.floor(material.bg.h * rate),
              scale: rate,
            },
            shadowBlur: options.shadow_show ? scaledMainHeight * ((options.shadow || 6) / 100) / rate : 0,
            cornerRadius: options.radius_show ? scaledMainHeight * ((options.radius || 2.1) / 100) / rate : 0,
          },
        }
      }, { inputPath, scenario })

      expect(browserResult.terminal).toBe('completed')
      const outputPath = path.join(outputDirectory, `${scenario.id}.jpg`)
      await expect.poll(() => fs.existsSync(outputPath), { timeout: 10_000 }).toBe(true)
      const outputMetadata = await sharp(outputPath).metadata()

      metadata[scenario.id] = browserResult.exif
      captured.push({
        id: scenario.id,
        input: `../input/${scenario.inputFile}`,
        inputSha256: sha256File(inputPath),
        options: scenario.options,
        templateKeys: scenario.templateKeys,
        expectedOutput: `output/${scenario.id}.jpg`,
        expectedMetadata: `metadata/${scenario.id}.json`,
        output: {
          width: outputMetadata.width ?? 0,
          height: outputMetadata.height ?? 0,
          density: outputMetadata.density ?? null,
        },
        exactGeometry: browserResult.geometry,
        perceptualThreshold: scenario.perceptualThreshold,
      })
    }

    return {
      manifest: {
        schemaVersion: 1,
        renderer: {
          name: 'electron-sharp-canvas',
          revision: execFileSync('git', ['rev-parse', 'origin/main'], { cwd: root, encoding: 'utf8' }).trim(),
          sharp: createRequire(import.meta.url)('sharp/package.json').version,
        },
        scenarios: captured,
      },
      metadata,
    }
  }
  finally {
    await app.close()
    fs.rmSync(fixtureHome, { recursive: true, force: true })
  }
}

async function installCaptureListeners(page: Page) {
  await page.evaluate(() => {
    const fixtureWindow = window as typeof window & {
      api: Record<string, (callback: (data: any) => void) => void>
      __yiyinFixtureEvents: {
        progress: { id: string, progress: number }[]
        failed: { id: string, msg: string }[]
        geometry: any[]
      }
    }
    fixtureWindow.__yiyinFixtureEvents = { progress: [], failed: [], geometry: [] }
    fixtureWindow.api['on:progress'](data => fixtureWindow.__yiyinFixtureEvents.progress.push(data))
    fixtureWindow.api['on:faildTask'](data => fixtureWindow.__yiyinFixtureEvents.failed.push(data))
    fixtureWindow.api['on:genMainImgShadow'](data => fixtureWindow.__yiyinFixtureEvents.geometry.push(data))
  })
}

function writeCapture(result: CaptureResult, metadataDirectory: string, manifestPath: string) {
  fs.mkdirSync(metadataDirectory, { recursive: true })
  fs.mkdirSync(path.dirname(manifestPath), { recursive: true })
  fs.writeFileSync(manifestPath, `${JSON.stringify(result.manifest, null, 2)}\n`)

  for (const [id, metadata] of Object.entries(result.metadata)) {
    fs.writeFileSync(path.join(metadataDirectory, `${id}.json`), `${JSON.stringify(metadata, null, 2)}\n`)
  }
}

function resetDirectory(directory: string) {
  fs.rmSync(directory, { recursive: true, force: true })
  fs.mkdirSync(directory, { recursive: true })
}

function sha256File(filePath: string) {
  return createHash('sha256').update(fs.readFileSync(filePath)).digest('hex')
}
