import { useState } from 'react'
import { Select } from '../../components/Select'
import type { ResourceDescriptorDto } from '../../platform/types'
import { FontDialog } from './FontDialog'

const DEFAULT_FONT = 'PingFang SC'
const BUNDLED_FONTS = [
  DEFAULT_FONT,
  '春风楷',
  '千图小兔',
  'FrederickatheGreat',
  'Neoneon',
] as const

interface FontSelectProps {
  value: string
  resources: ResourceDescriptorDto[]
  onChange(font: string): Promise<unknown>
  onRegister(name: string): Promise<ResourceDescriptorDto>
  onRemove(id: string): Promise<ResourceDescriptorDto[]>
}

export function FontSelect({
  value,
  resources,
  onChange,
  onRegister,
  onRemove,
}: FontSelectProps) {
  const [dialogOpen, setDialogOpen] = useState(false)
  const [error, setError] = useState('')
  const customFonts = resources.filter((resource) => resource.kind === 'font')
  const fonts = [
    ...BUNDLED_FONTS,
    ...customFonts.map((resource) => resource.displayName),
  ].filter((font, index, values) => values.indexOf(font) === index)

  const remove = async (resource: ResourceDescriptorDto) => {
    setError('')
    try {
      await onRemove(resource.id)
      if (value === resource.displayName) {
        await onChange(DEFAULT_FONT)
      }
    } catch (reason) {
      setError(readMessage(reason))
    }
  }

  return (
    <div className="font-select-wrap">
      <Select
        value={fonts.includes(value) ? value : DEFAULT_FONT}
        onValueChange={(font) => void onChange(font)}
        label="字体"
        action={{
          label: '添加字体',
          content: '+',
          onClick: () => setDialogOpen(true),
        }}
        options={fonts.map((font) => {
          const resource = customFonts.find(
            (candidate) => candidate.displayName === font,
          )
          return {
            value: font,
            label: font,
            action: resource
              ? {
                  label: `删除字体 ${resource.displayName}`,
                  content: '×',
                  onClick: () => void remove(resource),
                }
              : undefined,
          }
        })}
      />
      {error && <span role="alert">{error}</span>}
      <FontDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onRegister={onRegister}
      />
    </div>
  )
}

function readMessage(reason: unknown): string {
  return typeof reason === 'object' && reason !== null && 'message' in reason
    ? String(reason.message)
    : '字体操作失败'
}
