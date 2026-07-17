import { Select as BaseSelect } from '@base-ui/react/select'

interface SelectOption {
  value: string
  label: string
}

interface SelectProps {
  value: string
  onValueChange(value: string): void
  label: string
  options: SelectOption[]
}

export function Select({ value, onValueChange, label, options }: SelectProps) {
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
            <BaseSelect.List>
              {options.map((option) => (
                <BaseSelect.Item
                  key={option.value}
                  value={option.value}
                  className="select-item"
                >
                  <BaseSelect.ItemIndicator>✓</BaseSelect.ItemIndicator>
                  <BaseSelect.ItemText>{option.label}</BaseSelect.ItemText>
                </BaseSelect.Item>
              ))}
            </BaseSelect.List>
          </BaseSelect.Popup>
        </BaseSelect.Positioner>
      </BaseSelect.Portal>
    </BaseSelect.Root>
  )
}
