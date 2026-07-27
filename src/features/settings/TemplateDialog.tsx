import { useEffect, useState } from 'react'
import { Controller, useForm } from 'react-hook-form'
import { Dialog } from '../../components/Dialog'
import { RadioGroup } from '../../components/RadioGroup'
import { Switch } from '../../components/Switch'
import type {
  CaseConversionDto,
  TemplateDto,
  TemplateFieldDto,
  VerticalAlignDto,
} from '../../platform/types'

interface TemplateDialogProps {
  open: boolean
  template: TemplateDto | null
  fields: TemplateFieldDto[]
  onOpenChange(open: boolean): void
  onSave(template: TemplateDto): Promise<void>
}

interface TemplateFormValues {
  enabled: boolean
  name: string
  format: string
  verticalAlign: VerticalAlignDto
  height: string
  fontFamily: string
  fontSize: string
  fontBold: boolean
  fontItalic: boolean
  fontCase: CaseConversionDto
  fontColor: string
}

export function TemplateDialog({
  open,
  template,
  fields,
  onOpenChange,
  onSave,
}: TemplateDialogProps) {
  const [error, setError] = useState('')
  const form = useForm<TemplateFormValues>({
    defaultValues: templateValues(template),
  })

  useEffect(() => {
    if (open) {
      form.reset(templateValues(template))
      setError('')
    }
  }, [form, open, template])

  const submit = form.handleSubmit(async (values) => {
    const fontSize = Number(values.fontSize)
    const height = values.height.trim() ? Number(values.height) : null
    if (!values.name.trim() || !values.format.trim()) {
      setError('名称和模板内容不能为空')
      return
    }
    if (!Number.isFinite(fontSize) || fontSize <= 0) {
      setError('字体大小必须大于 0')
      return
    }
    if (height !== null && (!Number.isFinite(height) || height <= 0)) {
      setError('模板高度必须大于 0')
      return
    }
    const next: TemplateDto = {
      key: template?.key ?? `custom-template-${crypto.randomUUID()}`,
      name: values.name.trim(),
      format: values.format,
      enabled: values.enabled,
      kind: template?.kind ?? 'custom',
      height,
      font: {
        family: values.fontFamily,
        size: fontSize,
        bold: values.fontBold,
        italic: values.fontItalic,
        caseConversion: values.fontCase,
        color: values.fontColor,
      },
      verticalAlign: values.verticalAlign,
    }
    setError('')
    try {
      await onSave(next)
      onOpenChange(false)
    } catch (reason) {
      setError(readMessage(reason))
    }
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange} title="设置文字模板">
      <form className="settings-form" onSubmit={submit}>
        <Controller
          control={form.control}
          name="enabled"
          render={({ field }) => (
            <Switch
              checked={field.value}
              onCheckedChange={field.onChange}
              label="是否显示"
            />
          )}
        />
        <label>
          名称
          <input aria-label="名称" {...form.register('name')} />
        </label>
        <label>
          模板内容
          <input aria-label="模板内容" {...form.register('format')} />
        </label>
        <fieldset className="placeholder-actions">
          <legend>插入参数</legend>
          {fields.map((field) => (
            <button
              type="button"
              key={field.key}
              onClick={() =>
                form.setValue(
                  'format',
                  `${form.getValues('format')}{${field.key}}`,
                  { shouldDirty: true },
                )
              }
            >
              插入 {field.name}
            </button>
          ))}
        </fieldset>
        <Controller
          control={form.control}
          name="verticalAlign"
          render={({ field }) => (
            <RadioGroup
              value={field.value}
              onValueChange={field.onChange}
              label="图片对齐"
              options={[
                { value: 'baseline', label: '基线对齐' },
                { value: 'center', label: '居中对齐' },
              ]}
            />
          )}
        />
        <label>
          模板高度
          <input aria-label="模板高度" {...form.register('height')} />
        </label>
        <fieldset className="font-fields">
          <legend>字体参数</legend>
          <label>
            字体大小
            <input aria-label="模板字体大小" {...form.register('fontSize')} />
          </label>
          <label>
            字体
            <input aria-label="模板字体" {...form.register('fontFamily')} />
          </label>
          <label>
            字体颜色
            <input aria-label="模板字体颜色" {...form.register('fontColor')} />
          </label>
          <Controller
            control={form.control}
            name="fontBold"
            render={({ field }) => (
              <Switch
                checked={field.value}
                onCheckedChange={field.onChange}
                label="模板粗体"
              />
            )}
          />
          <Controller
            control={form.control}
            name="fontItalic"
            render={({ field }) => (
              <Switch
                checked={field.value}
                onCheckedChange={field.onChange}
                label="模板斜体"
              />
            )}
          />
          <Controller
            control={form.control}
            name="fontCase"
            render={({ field }) => (
              <RadioGroup
                value={field.value}
                onValueChange={field.onChange}
                label="模板格式化"
                options={[
                  { value: 'default', label: '默认' },
                  { value: 'uppercase', label: '大写' },
                  { value: 'lowercase', label: '小写' },
                ]}
              />
            )}
          />
        </fieldset>
        {error && <p role="alert">{error}</p>}
        <div className="dialog-actions">
          <button type="button" onClick={() => onOpenChange(false)}>
            取消
          </button>
          <button type="submit">保存</button>
        </div>
      </form>
    </Dialog>
  )
}

function templateValues(template: TemplateDto | null): TemplateFormValues {
  return {
    enabled: template?.enabled ?? false,
    name: template?.name ?? '',
    format: template?.format ?? '',
    verticalAlign: template?.verticalAlign ?? 'baseline',
    height:
      template?.height === null || template?.height === undefined
        ? ''
        : String(template.height),
    fontFamily: template?.font.family ?? '',
    fontSize: String(template?.font.size ?? 2.2),
    fontBold: template?.font.bold ?? false,
    fontItalic: template?.font.italic ?? false,
    fontCase: template?.font.caseConversion ?? 'default',
    fontColor: template?.font.color ?? '',
  }
}

function readMessage(reason: unknown): string {
  return typeof reason === 'object' && reason !== null && 'message' in reason
    ? String(reason.message)
    : '操作失败'
}
