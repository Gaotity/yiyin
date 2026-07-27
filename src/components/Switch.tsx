import { Switch as BaseSwitch } from '@base-ui/react/switch'

interface SwitchProps {
  checked: boolean
  onCheckedChange(checked: boolean): void
  label: string
  disabled?: boolean
}

export function Switch({
  checked,
  onCheckedChange,
  label,
  disabled = false,
}: SwitchProps) {
  return (
    <div className="switch-control">
      <BaseSwitch.Root
        checked={checked}
        disabled={disabled}
        onCheckedChange={onCheckedChange}
        aria-label={label}
      >
        <BaseSwitch.Thumb className="switch-thumb" />
      </BaseSwitch.Root>
      <span>{label}</span>
    </div>
  )
}
