import type { IpcMainInvokeEvent } from 'electron'
import { isTrustedRendererUrl } from '../security/urls'

interface SenderEvent {
  sender: { mainFrame: unknown }
  senderFrame: { url: string } | null
}

export function assertTrustedSender(event: SenderEvent | IpcMainInvokeEvent, isDevelopment: boolean) {
  if (!event.senderFrame || event.senderFrame !== event.sender.mainFrame) {
    throw new Error('IPC request must originate from the main frame')
  }

  if (!isTrustedRendererUrl(event.senderFrame.url, isDevelopment)) {
    throw new Error('IPC request origin is not trusted')
  }
}
