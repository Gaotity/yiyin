import { useState } from 'react'
import { Drawer } from '../../components/Drawer'
import { Switch } from '../../components/Switch'
import type { PublicConfigDto, TemplateDto } from '../../platform/types'
import { TemplateDialog } from './TemplateDialog'

interface TemplateDrawerProps {
  open: boolean
  config: PublicConfigDto
  onOpenChange(open: boolean): void
  onSave(config: PublicConfigDto): Promise<PublicConfigDto>
}

export function TemplateDrawer({
  open,
  config,
  onOpenChange,
  onSave,
}: TemplateDrawerProps) {
  const [editing, setEditing] = useState<TemplateDto | null | undefined>()

  const saveTemplate = async (template: TemplateDto) => {
    const next = structuredClone(config)
    const index = next.templates.findIndex(
      (candidate) => candidate.key === template.key,
    )
    if (index === -1) {
      next.templates.push(template)
    } else {
      next.templates[index] = template
    }
    await onSave(next)
  }

  const removeTemplate = async (template: TemplateDto) => {
    if (template.kind === 'system') {
      return
    }
    const next = structuredClone(config)
    next.templates = next.templates.filter(
      (candidate) => candidate.key !== template.key,
    )
    await onSave(next)
  }

  const moveTemplate = async (index: number, offset: -1 | 1) => {
    const destination = index + offset
    if (destination < 0 || destination >= config.templates.length) {
      return
    }
    const next = structuredClone(config)
    const current = next.templates[index]
    const adjacent = next.templates[destination]
    if (!current || !adjacent) {
      return
    }
    next.templates[index] = adjacent
    next.templates[destination] = current
    await onSave(next)
  }

  return (
    <>
      <Drawer open={open} onOpenChange={onOpenChange} title="文本模板设置">
        <div className="settings-list-heading">
          <h2>文本模板设置</h2>
          <button
            type="button"
            aria-label="添加文字模板"
            onClick={() => setEditing(null)}
          >
            +
          </button>
        </div>
        <section className="settings-list" aria-label="文字模板">
          {config.templates.map((template, index) => (
            <div className="settings-list-row" key={template.key}>
              <Switch
                checked={template.enabled}
                onCheckedChange={(enabled) =>
                  void saveTemplate({ ...template, enabled })
                }
                label={`显示${template.name}`}
              />
              <span>{template.name}</span>
              <button
                type="button"
                aria-label={`上移${template.name}`}
                disabled={index === 0}
                onClick={() => void moveTemplate(index, -1)}
              >
                ↑
              </button>
              <button
                type="button"
                aria-label={`下移${template.name}`}
                disabled={index === config.templates.length - 1}
                onClick={() => void moveTemplate(index, 1)}
              >
                ↓
              </button>
              <button
                type="button"
                aria-label={`编辑${template.name}`}
                onClick={() => setEditing(template)}
              >
                编辑
              </button>
              {template.kind === 'custom' && (
                <button
                  type="button"
                  aria-label={`删除${template.name}`}
                  onClick={() => void removeTemplate(template)}
                >
                  删除
                </button>
              )}
            </div>
          ))}
        </section>
      </Drawer>
      <TemplateDialog
        open={editing !== undefined}
        template={editing ?? null}
        fields={[...config.templateFields, ...config.customTemplateFields]}
        onOpenChange={(dialogOpen) => {
          if (!dialogOpen) {
            setEditing(undefined)
          }
        }}
        onSave={saveTemplate}
      />
    </>
  )
}
