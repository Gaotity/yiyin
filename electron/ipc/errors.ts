import type { PlatformErrorCode, PlatformResult } from '@common/platform/result'

export class PlatformBoundaryError extends Error {
  constructor(public readonly code: PlatformErrorCode, message: string) {
    super(message)
    this.name = 'PlatformBoundaryError'
  }
}

export function success<T>(data: T): PlatformResult<T> {
  return { ok: true, data }
}

export function failure(error: unknown): PlatformResult<never> {
  if (error instanceof PlatformBoundaryError) {
    return { ok: false, error: { code: error.code, message: error.message } }
  }
  return { ok: false, error: { code: 'INTERNAL', message: 'The operation could not be completed' } }
}
