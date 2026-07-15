import path from 'node:path'

export function resolveProtocolPath(root: string, encodedPathname: string) {
  let pathname: string
  try {
    pathname = decodeURIComponent(encodedPathname)
  }
  catch {
    throw new Error('Protocol path is invalid')
  }

  if (pathname.includes('\0')) {
    throw new Error('Protocol path is invalid')
  }

  const normalizedRoot = path.resolve(root)
  const candidate = path.resolve(normalizedRoot, `.${pathname}`)
  const relative = path.relative(normalizedRoot, candidate)

  if (relative.startsWith('..') || path.isAbsolute(relative)) {
    throw new Error('Protocol path resolves outside the application root')
  }

  return candidate
}
