import { useEffect, useState } from 'react'
import { Controller, useForm } from 'react-hook-form'
import { Dialog } from '../../components/Dialog'
import { RadioGroup } from '../../components/RadioGroup'
import { Switch } from '../../components/Switch'
import type {
  CaseConversionDto,
  ResourceDescriptorDto,
  TemplateFieldDto,
} from '../../platform/types'

interface FieldDialogProps {
  open: boolean
  field: TemplateFieldDto | null
  custom: boolean
  onOpenChange(open: boolean): void
  onRegisterOverlay(): Promise<ResourceDescriptorDto>
  onSave(field: TemplateFieldDto): Promise<void>
}

interface FieldFormValues {
  name: string
  visible: boolean
  useCustomValue: boolean
  forceCustomValue: boolean
  customValue: string
  contentKind: 'text' | 'image'
  darkImageId: string
  lightImageId: string
  fontEnabled: boolean
  fontFamily: string
  fontSize: string
  fontBold: boolean
  fontItalic: boolean
  fontCase: CaseConversionDto
  fontColor: string
}

export function FieldDialog({
  open,
  field,
  custom,
  onOpenChange,
  onRegisterOverlay,
  onSave,
}: FieldDialogProps) {
  const [error, setError] = useState('')
  const form = useForm<FieldFormValues>({ defaultValues: fieldValues(field) })
  const contentKind = form.watch('contentKind')
  const fontEnabled = form.watch('fontEnabled')

  useEffect(() => {
    if (open) {
      form.reset(fieldValues(field))
      setError('')
    }
  }, [field, form, open])

  const registerImage = async (kind: 'darkImageId' | 'lightImageId') => {
    setError('')
    try {
      const resource = await onRegisterOverlay()
      form.setValue(kind, resource.id, { shouldDirty: true })
    } catch (reason) {
      setError(readMessage(reason))
    }
  }

  const submit = form.handleSubmit(async (values) => {
    const fontSize = Number(values.fontSize)
    if (values.fontEnabled && (!Number.isFinite(fontSize) || fontSize <= 0)) {
      setError('字体大小必须大于 0')
      return
    }
    if (custom && !values.name.trim()) {
      setError('名称不能为空')
      return
    }
    const next: TemplateFieldDto = {
      key: field?.key ?? `custom-field-${crypto.randomUUID()}`,
      name: custom ? values.name.trim() : (field?.name ?? values.name.trim()),
      visible: values.visible,
      useCustomValue: values.contentKind === 'text' && values.useCustomValue,
      forceCustomValue:
        values.contentKind === 'text' && values.forceCustomValue,
      customValue: values.contentKind === 'text' ? values.customValue : '',
      contentKind: values.contentKind,
      darkImageId:
        values.contentKind === 'image' && values.darkImageId
          ? values.darkImageId
          : null,
      lightImageId:
        values.contentKind === 'image' && values.lightImageId
          ? values.lightImageId
          : null,
      fontOverride: values.fontEnabled
        ? {
            family: values.fontFamily,
            size: fontSize,
            bold: values.fontBold,
            italic: values.fontItalic,
            caseConversion: values.fontCase,
            color: values.fontColor,
          }
        : null,
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
    <Dialog
      open={open}
      onOpenChange={onOpenChange}
      title={field ? `设置${field.name}` : '添加参数'}
    >
      <form className="settings-form" onSubmit={submit}>
        <Controller
          control={form.control}
          name="visible"
          render={({ field: control }) => (
            <Switch
              checked={control.value}
              onCheckedChange={control.onChange}
              label="是否显示"
            />
          )}
        />
        {custom && (
          <label>
            名称
            <input aria-label="字段名称" {...form.register('name')} />
          </label>
        )}
        <Controller
          control={form.control}
          name="contentKind"
          render={({ field: control }) => (
            <RadioGroup
              value={control.value}
              onValueChange={control.onChange}
              label="类型"
              options={[
                { value: 'text', label: '文本' },
                { value: 'image', label: '图片' },
              ]}
            />
          )}
        />
        {contentKind === 'text' ? (
          <>
            <Controller
              control={form.control}
              name="useCustomValue"
              render={({ field: control }) => (
                <Switch
                  checked={control.value}
                  onCheckedChange={control.onChange}
                  label="是否生效"
                />
              )}
            />
            <Controller
              control={form.control}
              name="forceCustomValue"
              render={({ field: control }) => (
                <Switch
                  checked={control.value}
                  onCheckedChange={control.onChange}
                  label="强制替换"
                />
              )}
            />
            <label>
              文本内容
              <input aria-label="文本内容" {...form.register('customValue')} />
            </label>
          </>
        ) : (
          <div className="overlay-actions">
            <button
              type="button"
              onClick={() => void registerImage('lightImageId')}
            >
              选择模糊背景图片
            </button>
            {form.watch('lightImageId') && <span>已选择模糊背景图片</span>}
            <button
              type="button"
              onClick={() => void registerImage('darkImageId')}
            >
              选择纯色背景图片
            </button>
            {form.watch('darkImageId') && <span>已选择纯色背景图片</span>}
          </div>
        )}
        <Controller
          control={form.control}
          name="fontEnabled"
          render={({ field: control }) => (
            <Switch
              checked={control.value}
              onCheckedChange={control.onChange}
              label="自定义字体参数"
            />
          )}
        />
        {fontEnabled && (
          <FontFields register={form.register} control={form.control} />
        )}
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

function FontFields({
  register,
  control,
}: Pick<ReturnType<typeof useForm<FieldFormValues>>, 'register' | 'control'>) {
  return (
    <fieldset className="font-fields">
      <legend>字体参数</legend>
      <label>
        字体大小
        <input aria-label="字体大小" {...register('fontSize')} />
      </label>
      <label>
        字体
        <input aria-label="字段字体" {...register('fontFamily')} />
      </label>
      <label>
        字体颜色
        <input aria-label="字体颜色" {...register('fontColor')} />
      </label>
      <Controller
        control={control}
        name="fontBold"
        render={({ field }) => (
          <Switch
            checked={field.value}
            onCheckedChange={field.onChange}
            label="粗体"
          />
        )}
      />
      <Controller
        control={control}
        name="fontItalic"
        render={({ field }) => (
          <Switch
            checked={field.value}
            onCheckedChange={field.onChange}
            label="斜体"
          />
        )}
      />
      <Controller
        control={control}
        name="fontCase"
        render={({ field }) => (
          <RadioGroup
            value={field.value}
            onValueChange={field.onChange}
            label="格式化"
            options={[
              { value: 'default', label: '默认' },
              { value: 'uppercase', label: '大写' },
              { value: 'lowercase', label: '小写' },
            ]}
          />
        )}
      />
    </fieldset>
  )
}

function fieldValues(field: TemplateFieldDto | null): FieldFormValues {
  return {
    name: field?.name ?? '',
    visible: field?.visible ?? true,
    useCustomValue: field?.useCustomValue ?? false,
    forceCustomValue: field?.forceCustomValue ?? false,
    customValue: field?.customValue ?? '',
    contentKind: field?.contentKind ?? 'text',
    darkImageId: field?.darkImageId ?? '',
    lightImageId: field?.lightImageId ?? '',
    fontEnabled:
      field?.fontOverride !== null && field?.fontOverride !== undefined,
    fontFamily: field?.fontOverride?.family ?? '',
    fontSize: String(field?.fontOverride?.size ?? 2.2),
    fontBold: field?.fontOverride?.bold ?? false,
    fontItalic: field?.fontOverride?.italic ?? false,
    fontCase: field?.fontOverride?.caseConversion ?? 'default',
    fontColor: field?.fontOverride?.color ?? '',
  }
}

function readMessage(reason: unknown): string {
  return typeof reason === 'object' && reason !== null && 'message' in reason
    ? String(reason.message)
    : '操作失败'
}
