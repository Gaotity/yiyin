import type { ResourceRegistry } from '../resources/registry'
import fs from 'node:fs/promises'
import path from 'node:path'
import { net, protocol } from 'electron'
import { resolveProtocolPath } from '../security/protocol-path'

const MIME_TYPES: Record<string, string> = {
  '.css': 'text/css; charset=utf-8',
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.json': 'application/json; charset=utf-8',
  '.jpg': 'image/jpeg',
  '.jpeg': 'image/jpeg',
  '.otf': 'font/otf',
  '.png': 'image/png',
  '.svg': 'image/svg+xml',
  '.ttf': 'font/ttf',
  '.webp': 'image/webp',
}

const CSP = [
  'default-src \'self\'',
  'script-src \'self\'',
  'style-src \'self\' \'unsafe-inline\'',
  'img-src \'self\' data: yiyin:',
  'font-src \'self\' data: yiyin:',
  'connect-src \'self\'',
  'object-src \'none\'',
  'base-uri \'none\'',
  'frame-ancestors \'none\'',
  'form-action \'none\'',
].join('; ')

protocol.registerSchemesAsPrivileged([{
  scheme: 'yiyin',
  privileges: {
    standard: true,
    secure: true,
    supportFetchAPI: true,
    corsEnabled: true,
    stream: true,
  },
}])

export function registerYiyinProtocol(webRoot: string, registry: ResourceRegistry) {
  protocol.handle('yiyin', async (request) => {
    try {
      const url = new URL(request.url)
      let filePath: string
      if (url.hostname === 'app') filePath = resolveProtocolPath(webRoot, url.pathname)
      else if (url.hostname === 'resource') filePath = registry.resolve(url.pathname.slice(1))
      else return new Response('Not found', { status: 404 })

      const body = await fs.readFile(filePath)
      const headers = new Headers({
        'Content-Type': MIME_TYPES[path.extname(filePath).toLowerCase()] ?? 'application/octet-stream',
        'Content-Security-Policy': CSP,
        'Cross-Origin-Resource-Policy': 'same-site',
        'X-Content-Type-Options': 'nosniff',
      })
      if (url.hostname === 'resource') headers.set('Access-Control-Allow-Origin', 'yiyin://app')
      return new Response(body, { status: 200, headers })
    }
    catch {
      return new Response('Not found', { status: 404 })
    }
  })

  return () => protocol.unhandle('yiyin')
}

export async function canLoadApplicationUrl(url: string) {
  const response = await net.fetch(url)
  return response.ok
}
