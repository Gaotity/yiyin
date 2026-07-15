const DEFAULT_EXTERNAL_HOSTS = new Set(['github.com'])

export function isAllowedExternalUrl(rawUrl: string, allowedHosts = DEFAULT_EXTERNAL_HOSTS) {
  try {
    const url = new URL(rawUrl)
    return url.protocol === 'https:' && allowedHosts.has(url.hostname)
  }
  catch {
    return false
  }
}

export function isTrustedRendererUrl(rawUrl: string, isDevelopment: boolean) {
  try {
    const url = new URL(rawUrl)
    if (isDevelopment) {
      return url.origin === 'http://127.0.0.1:5173'
        && url.pathname.startsWith('/main/')
    }

    return url.protocol === 'yiyin:'
      && url.hostname === 'app'
      && url.pathname.startsWith('/main/')
  }
  catch {
    return false
  }
}
