import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { resolveProtocolPath } from '../../electron/security/protocol-path'

describe('custom protocol containment', () => {
  const root = path.resolve('/application/web')

  it('resolves an application asset inside the root', () => {
    expect(resolveProtocolPath(root, '/main/index.html')).toBe(path.join(root, 'main/index.html'))
  })

  it('rejects traversal, encoded traversal, and NUL bytes', () => {
    expect(() => resolveProtocolPath(root, '/../secret')).toThrow('outside')
    expect(() => resolveProtocolPath(root, '/%2e%2e/secret')).toThrow('outside')
    expect(() => resolveProtocolPath(root, '/main/%00index.html')).toThrow('invalid')
    expect(() => resolveProtocolPath(root, '/main/%E0%A4%A')).toThrow('invalid')
  })
})
