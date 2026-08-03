import { useState } from 'react'
import { Drawer } from '../../components/Drawer'
import { Switch } from '../../components/Switch'
import type {
  PublicConfigDto,
  ResourceDescriptorDto,
  TemplateFieldDto,
} from '../../platform/types'
import { removeCustomTemplateField, upsertTemplateField } from './configIntents'
import { FieldDialog } from './FieldDialog'

interface FieldDrawerProps {
  open: boolean
  config: PublicConfigDto
  onOpenChange(open: boolean): void
  onSave(config: PublicConfigDto): Promise<PublicConfigDto>
  onRegisterOverlay(): Promise<ResourceDescriptorDto>
}

interface EditingField {
  field: TemplateFieldDto | null
  custom: boolean
}

export function FieldDrawer({
  open,
  config,
  onOpenChange,
  onSave,
  onRegisterOverlay,
}: FieldDrawerProps) {
  const [editing, setEditing] = useState<EditingField | null>(null)

  const updateField = async (field: TemplateFieldDto, custom: boolean) => {
    await onSave(upsertTemplateField(config, field, custom))
  }

  const removeCustom = async (key: string) => {
    await onSave(removeCustomTemplateField(config, key))
  }

  return (
    <>
      <Drawer open={open} onOpenChange={onOpenChange} title="参数设置">
        <section className="settings-list" aria-label="相机参数">
          <h2>相机参数</h2>
          {config.templateFields.map((field) => (
            <FieldRow
              key={field.key}
              field={field}
              onVisible={(visible) =>
                void updateField({ ...field, visible }, false)
              }
              onEdit={() => setEditing({ field, custom: false })}
            />
          ))}
        </section>
        <section className="settings-list" aria-label="自定义参数">
          <div className="settings-list-heading">
            <h2>自定义参数</h2>
            <button
              type="button"
              aria-label="添加自定义参数"
              onClick={() => setEditing({ field: null, custom: true })}
            >
              +
            </button>
          </div>
          {config.customTemplateFields.map((field) => (
            <FieldRow
              key={field.key}
              field={field}
              onVisible={(visible) =>
                void updateField({ ...field, visible }, true)
              }
              onEdit={() => setEditing({ field, custom: true })}
              onDelete={() => void removeCustom(field.key)}
            />
          ))}
        </section>
      </Drawer>
      <FieldDialog
        open={editing !== null}
        field={editing?.field ?? null}
        custom={editing?.custom ?? false}
        onOpenChange={(dialogOpen) => {
          if (!dialogOpen) {
            setEditing(null)
          }
        }}
        onRegisterOverlay={onRegisterOverlay}
        onSave={(field) => updateField(field, editing?.custom ?? false)}
      />
    </>
  )
}

interface FieldRowProps {
  field: TemplateFieldDto
  onVisible(visible: boolean): void
  onEdit(): void
  onDelete?(): void
}

function FieldRow({ field, onVisible, onEdit, onDelete }: FieldRowProps) {
  return (
    <div className="settings-list-row">
      <Switch
        checked={field.visible}
        onCheckedChange={onVisible}
        label={`显示${field.name}`}
      />
      <span>{field.name}</span>
      <span className="settings-value">
        {field.contentKind === 'text' ? field.customValue : '图片'}
      </span>
      <button type="button" aria-label={`编辑${field.name}`} onClick={onEdit}>
        编辑
      </button>
      {onDelete && (
        <button
          type="button"
          aria-label={`删除${field.name}`}
          onClick={onDelete}
        >
          删除
        </button>
      )}
    </div>
  )
}
