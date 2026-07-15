import type { PlatformBridge } from '@common/platform/bridge'

declare global {
  interface Window {
    platform: PlatformBridge
  }
}

export {}
