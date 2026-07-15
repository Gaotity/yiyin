export type PlatformErrorCode
  = | 'CANCELLED'
    | 'CONFIG_INVALID'
    | 'FILE_INVALID'
    | 'FILE_NOT_FOUND'
    | 'FORBIDDEN'
    | 'INTERNAL'
    | 'INVALID_REQUEST'
    | 'RESOURCE_NOT_FOUND'
    | 'TASK_NOT_FOUND'

export interface PlatformError {
  code: PlatformErrorCode
  message: string
}

export type PlatformResult<T>
  = | { ok: true, data: T }
    | { ok: false, error: PlatformError }
