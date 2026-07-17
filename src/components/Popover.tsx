import { Popover as BasePopover } from '@base-ui/react/popover'
import type { ReactNode } from 'react'

interface PopoverProps {
  label: string
  trigger: ReactNode
  children: ReactNode
  className?: string
}

export function Popover({
  label,
  trigger,
  children,
  className = '',
}: PopoverProps) {
  return (
    <BasePopover.Root>
      <BasePopover.Trigger className="icon-button" aria-label={label}>
        {trigger}
      </BasePopover.Trigger>
      <BasePopover.Portal>
        <BasePopover.Positioner
          sideOffset={8}
          align="start"
          className="popover-positioner"
        >
          <BasePopover.Popup className={`popover-surface ${className}`}>
            {children}
          </BasePopover.Popup>
        </BasePopover.Positioner>
      </BasePopover.Portal>
    </BasePopover.Root>
  )
}
