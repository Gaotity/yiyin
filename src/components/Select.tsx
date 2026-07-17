import { Select as BaseSelect } from '@base-ui/react/select'
import type { ReactNode } from 'react'

interface SelectOption {
  value: string
  label: string
  action?: {
    label: string
    content: ReactNode
    onClick(): void
  }
}

interface SelectAction {
  label: string
  content: ReactNode
  onClick(): void
}

interface SelectProps {
  value: string
  onValueChange(value: string): void
  label: string
  options: SelectOption[]
  action?: SelectAction
}

export function Select({
  value,
  onValueChange,
  label,
  options,
  action,
}: SelectProps) {
  return (
    <BaseSelect.Root
      value={value}
      onValueChange={(next) => next && onValueChange(next)}
      items={options}
    >
      <BaseSelect.Trigger className="select-trigger" aria-label={label}>
        <BaseSelect.Value />
        <BaseSelect.Icon>⌄</BaseSelect.Icon>
      </BaseSelect.Trigger>
      <BaseSelect.Portal>
        <BaseSelect.Positioner className="popover-positioner" sideOffset={4}>
          <BaseSelect.Popup className="select-surface">
            {action && (
              <button
                type="button"
                className="select-popup-action"
                aria-label={action.label}
                onClick={action.onClick}
              >
                {action.content}
              </button>
            )}
            <BaseSelect.List>
              {options.map((option) => (
                <BaseSelect.Item
                  key={option.value}
                  value={option.value}
                  className="select-item"
                >
                  <BaseSelect.ItemIndicator>✓</BaseSelect.ItemIndicator>
                  <BaseSelect.ItemText>{option.label}</BaseSelect.ItemText>
                  {option.action && (
                    <button
                      type="button"
                      className="select-item-action"
                      aria-label={option.action.label}
                      onPointerDown={(event) => {
                        event.preventDefault()
                        event.stopPropagation()
                      }}
                      onClick={(event) => {
                        event.preventDefault()
                        event.stopPropagation()
                        option.action?.onClick()
                      }}
                    >
                      {option.action.content}
                    </button>
                  )}
                </BaseSelect.Item>
              ))}
            </BaseSelect.List>
          </BaseSelect.Popup>
        </BaseSelect.Positioner>
      </BaseSelect.Portal>
    </BaseSelect.Root>
  )
}
