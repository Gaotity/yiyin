import { _electron as electron, expect, test } from '@playwright/test'

test('renderer starts inside the hardened platform boundary', async () => {
  const application = await electron.launch({ args: ['.'] })
  try {
    const page = await application.firstWindow()
    await expect(page.locator('#app')).toBeVisible()
    const boundary = await page.evaluate(() => ({
      protocol: location.protocol,
      hasNodeProcess: typeof (globalThis as { process?: unknown }).process !== 'undefined',
      hasRequire: typeof (globalThis as { require?: unknown }).require !== 'undefined',
      hasLegacyApi: 'api' in window,
      namespaces: Object.keys((globalThis as unknown as { platform: Record<string, unknown> }).platform).sort(),
      csp: document.querySelector('meta[http-equiv="Content-Security-Policy"]')?.getAttribute('content'),
    }))

    expect(boundary).toMatchObject({
      protocol: 'yiyin:',
      hasNodeProcess: false,
      hasRequire: false,
      hasLegacyApi: false,
      namespaces: ['app', 'config', 'events', 'files', 'tasks'],
    })
    expect(boundary.csp).toContain('script-src \'self\'')
  }
  finally {
    await application.close()
  }
})
