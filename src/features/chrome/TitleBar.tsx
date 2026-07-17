import { Select } from '../../components/Select'
import type {
  ExternalDestinationDto,
  PublicConfigDto,
  ResourceDescriptorDto,
} from '../../platform/types'
import { FeedbackPopover } from './FeedbackPopover'
import { HelpPopover } from './HelpPopover'

interface TitleBarProps {
  config: PublicConfigDto
  resources: ResourceDescriptorDto[]
  onFontChange(font: string): void
  onReset(): void
  onMinimize(): void
  onClose(): void
  onOpen(destination: ExternalDestinationDto): void
}

export function TitleBar({
  config,
  resources,
  onFontChange,
  onReset,
  onMinimize,
  onClose,
  onOpen,
}: TitleBarProps) {
  const fonts = [
    config.options.font,
    ...resources
      .filter((resource) => resource.kind === 'font')
      .map((resource) => resource.displayName),
  ].filter((font, index, values) => values.indexOf(font) === index)

  return (
    <header className="title-bar" data-tauri-drag-region>
      <div className="title-bar-left">
        <FeedbackPopover resources={resources} onOpen={onOpen} />
        <HelpPopover />
      </div>
      <div className="title-bar-right">
        <Select
          value={config.options.font}
          onValueChange={onFontChange}
          label="字体"
          options={fonts.map((font) => ({ value: font, label: font }))}
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
