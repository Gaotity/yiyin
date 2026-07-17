import { resolve } from 'node:path'
import { pathToFileURL } from 'node:url'

const elementKey = 'element-6066-11e4-a52e-4f735466cecf'

export function createW3cClient(baseUrl = 'http://127.0.0.1:4444') {
  return async function command(method, path, body) {
    const response = await fetch(`${baseUrl}${path}`, {
      method,
      headers: { 'content-type': 'application/json' },
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: AbortSignal.timeout(30_000),
    })
    const payload = await response.json()
    if (!response.ok || payload.value?.error) {
      throw new Error(
        payload.value?.message ??
          `WebDriver request failed: ${response.status}`,
      )
    }
    return payload
  }
}

export async function runDesktopSmoke({ application, baseUrl }) {
  const command = createW3cClient(baseUrl)
  const session = await waitFor(
    () =>
      command('POST', '/session', {
        capabilities: {
          alwaysMatch: {
            'tauri:options': { application: resolve(application) },
          },
        },
      }),
    30_000,
  )
  const sessionId = session.value.sessionId ?? session.sessionId
  if (!sessionId) {
    throw new Error('tauri-driver returned no session ID')
  }
  const endpoint = `/session/${sessionId}`
  try {
    const title = await command('GET', `${endpoint}/title`)
    if (title.value !== '壹印') {
      throw new Error(`Unexpected window title: ${String(title.value)}`)
    }
    const rect = await command('GET', `${endpoint}/window/rect`)
    if (rect.value.width !== 900 || rect.value.height !== 730) {
      throw new Error(
        `Unexpected window size: ${rect.value.width}x${rect.value.height}`,
      )
    }

    await waitForElement(command, endpoint, '[aria-label="图片任务"]')
    await clickXpath(
      command,
      endpoint,
      '//button[normalize-space()="参数设置"]',
    )
    await waitForElement(command, endpoint, '[aria-label="相机参数"]')
    await clickXpath(command, endpoint, '//button[normalize-space()="关闭"]')
    await clickXpath(
      command,
      endpoint,
      '//button[normalize-space()="模板设置"]',
    )
    await waitForElement(command, endpoint, '[aria-label="文字模板"]')
    await clickXpath(command, endpoint, '//button[normalize-space()="关闭"]')
    await clickCss(command, endpoint, '[aria-label="实时预览"]')
    await waitForElement(command, endpoint, 'img[alt="预览图"]', 30_000)
    await clickXpath(
      command,
      endpoint,
      '//button[normalize-space()="生成印框"]',
    )
    await waitForXpath(
      command,
      endpoint,
      '//*[normalize-space()="输出完成"]',
      60_000,
    )
  } finally {
    await command('DELETE', endpoint)
  }
}

async function clickCss(command, endpoint, selector) {
  const elementId = await element(command, endpoint, 'css selector', selector)
  await command('POST', `${endpoint}/element/${elementId}/click`, {})
}

async function clickXpath(command, endpoint, selector) {
  const elementId = await element(command, endpoint, 'xpath', selector)
  await command('POST', `${endpoint}/element/${elementId}/click`, {})
}

async function element(command, endpoint, using, value) {
  const response = await command('POST', `${endpoint}/element`, {
    using,
    value,
  })
  const id = response.value?.[elementKey]
  if (!id) {
    throw new Error(`WebDriver returned no element for ${value}`)
  }
  return id
}

async function waitForElement(command, endpoint, selector, timeout = 10_000) {
  return waitFor(
    () => element(command, endpoint, 'css selector', selector),
    timeout,
  )
}

async function waitForXpath(command, endpoint, selector, timeout) {
  return waitFor(() => element(command, endpoint, 'xpath', selector), timeout)
}

async function waitFor(operation, timeout) {
  const deadline = Date.now() + timeout
  let lastError
  while (Date.now() < deadline) {
    try {
      return await operation()
    } catch (error) {
      lastError = error
      await new Promise((resolvePromise) => setTimeout(resolvePromise, 250))
    }
  }
  throw lastError ?? new Error('WebDriver wait timed out')
}

function parseArgs(argv) {
  const options = { baseUrl: 'http://127.0.0.1:4444' }
  for (let index = 0; index < argv.length; index += 2) {
    const key = argv[index]
    const value = argv[index + 1]
    if (!key?.startsWith('--') || !value) {
      throw new Error('Expected --application <path> [--base-url <url>]')
    }
    if (key === '--application') {
      options.application = value
    } else if (key === '--base-url') {
      options.baseUrl = value
    } else {
      throw new Error(`Unknown option: ${key}`)
    }
  }
  if (!options.application) {
    throw new Error('Expected --application <path>')
  }
  return options
}

if (
  process.argv[1] &&
  import.meta.url === pathToFileURL(process.argv[1]).href
) {
  await runDesktopSmoke(parseArgs(process.argv.slice(2)))
}
