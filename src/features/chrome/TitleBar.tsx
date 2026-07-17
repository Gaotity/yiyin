import type {
  ExternalDestinationDto,
  PublicConfigDto,
  ResourceDescriptorDto,
} from '../../platform/types'
import { FontSelect } from '../settings/FontSelect'
import { FeedbackPopover } from './FeedbackPopover'
import { HelpPopover } from './HelpPopover'

interface TitleBarProps {
  config: PublicConfigDto
  resources: ResourceDescriptorDto[]
  onFontChange(font: string): Promise<unknown>
  onRegisterFont(name: string): Promise<ResourceDescriptorDto>
  onRemoveFont(id: string): Promise<ResourceDescriptorDto[]>
  onReset(): void
  onMinimize(): void
  onClose(): void
  onOpen(destination: ExternalDestinationDto): void
}

export function TitleBar({
  config,
  resources,
  onFontChange,
  onRegisterFont,
  onRemoveFont,
  onReset,
  onMinimize,
  onClose,
  onOpen,
}: TitleBarProps) {
  return (
    <header className="title-bar" data-tauri-drag-region>
      <div className="title-bar-left">
        <FeedbackPopover resources={resources} onOpen={onOpen} />
        <HelpPopover />
      </div>
      <div className="title-bar-right">
        <FontSelect
          value={config.options.font}
          resources={resources}
          onChange={onFontChange}
          onRegister={onRegisterFont}
          onRemove={onRemoveFont}
        />
        <button
          type="button"
          className="icon-button"
          aria-label="重置回默认选项"
          onClick={onReset}
        >
          ↶
        </button>
        <button
          type="button"
          className="icon-button"
          aria-label="最小化"
          onClick={onMinimize}
        >
          −
        </button>
        <button
          type="button"
          className="icon-button"
          aria-label="关闭"
          onClick={onClose}
        >
          ×
        </button>
      </div>
    </header>
  )
}
