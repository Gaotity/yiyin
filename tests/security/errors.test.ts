import { describe, expect, it } from 'vitest'
import { failure, PlatformBoundaryError, success } from '../../electron/ipc/errors'

describe('typed platform errors', () => {
  it('returns stable success and known boundary errors', () => {
    expect(success('value')).toEqual({ ok: true, data: 'value' })
    expect(failure(new PlatformBoundaryError('FORBIDDEN', 'Denied'))).toEqual({
      ok: false,
      error: { code: 'FORBIDDEN', message: 'Denied' },
    })
  })

  it('redacts unknown errors', () => {
    const result = failure(new Error('/Users/private/secret'))
    expect(result).toEqual({
      ok: false,
      error: { code: 'INTERNAL', message: 'The operation could not be completed' },
    })
    expect(JSON.stringify(result)).not.toContain('/Users')
  })
})
