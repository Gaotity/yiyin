import { Radio } from '@base-ui/react/radio'
import { RadioGroup as BaseRadioGroup } from '@base-ui/react/radio-group'

interface RadioOption {
  value: string
  label: string
}

interface RadioGroupProps {
  value: string
  onValueChange(value: string): void
  label: string
  options: RadioOption[]
}

export function RadioGroup({
  value,
  onValueChange,
  label,
  options,
}: RadioGroupProps) {
  return (
    <BaseRadioGroup
      aria-label={label}
      className="radio-group"
      value={value}
      onValueChange={onValueChange}
    >
      {options.map((option) => (
        <div key={option.value} className="radio-control">
          <Radio.Root value={option.value} aria-label={option.label}>
            <Radio.Indicator className="radio-indicator" />
          </Radio.Root>
          {option.label}
        </div>
      ))}
    </BaseRadioGroup>
  )
}
