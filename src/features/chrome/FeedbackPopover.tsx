import { useState } from 'react'
import { Popover } from '../../components/Popover'
import type {
  ExternalDestinationDto,
  ResourceDescriptorDto,
} from '../../platform/types'

const QQ_GROUP = '718615618'

interface FeedbackPopoverProps {
  resources: ResourceDescriptorDto[]
  onOpen(destination: ExternalDestinationDto): void
}

export function FeedbackPopover({ resources, onOpen }: FeedbackPopoverProps) {
  const [copied, setCopied] = useState(false)
  const wechat = resources.find(
    (resource) => resource.displayName === 'zs-wx.jpg',
  )
  const alipay = resources.find(
    (resource) => resource.displayName === 'zs-zfb.jpg',
  )

  const copyGroup = async () => {
    await navigator.clipboard.writeText(QQ_GROUP)
    setCopied(true)
  }

  return (
    <Popover label="反馈与赞赏" trigger="☆" className="feedback-popover">
      <button
        type="button"
        className="popover-action"
        onClick={() => void copyGroup()}
      >
        QQ交流群:{QQ_GROUP}
      </button>
      {copied && <p role="status">群号已复制到粘贴板</p>}
      <button
        type="button"
        className="popover-action"
        onClick={() => onOpen('bilibiliFeedback')}
      >
        反馈 - 建议(B站私信)
      </button>
      <button
        type="button"
        className="popover-action"
        onClick={() => onOpen('issues')}
      >
        反馈 - 建议(Github Issues)
      </button>
      <strong>๑乛◡乛๑你不会想白嫖吧</strong>
      <div className="donation-codes">
        {wechat && (
          <figure>
            <img src={wechat.url} alt="微信赞赏码" />
            <figcaption>微信</figcaption>
          </figure>
        )}
        {alipay && (
          <figure>
            <img src={alipay.url} alt="支付宝赞赏码" />
            <figcaption>支付宝</figcaption>
          </figure>
        )}
      </div>
    </Popover>
  )
}
