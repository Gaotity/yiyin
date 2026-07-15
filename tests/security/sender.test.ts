import { describe, expect, it } from 'vitest'
import { assertTrustedSender } from '../../electron/ipc/sender'

function event(url: string, sameFrame = true) {
  const mainFrame = { url }
  return {
    sender: { mainFrame },
    senderFrame: sameFrame ? mainFrame : { url },
  }
}

describe('iPC sender validation', () => {
  it('accepts the trusted main frame', () => {
    expect(() => assertTrustedSender(event('yiyin://app/main/index.html'), false)).not.toThrow()
  })

  it('rejects subframes and hostile origins', () => {
    expect(() => assertTrustedSender(event('yiyin://app/main/index.html', false), false)).toThrow('main frame')
    expect(() => assertTrustedSender(event('https://evil.example'), false)).toThrow('origin')
  })
})
