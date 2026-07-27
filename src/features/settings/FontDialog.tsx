import { useEffect, useState } from 'react'
import { useForm } from 'react-hook-form'
import { Dialog } from '../../components/Dialog'
import type { ResourceDescriptorDto } from '../../platform/types'

interface FontDialogProps {
  open: boolean
  onOpenChange(open: boolean): void
  onRegister(name: string): Promise<ResourceDescriptorDto>
}

interface FontFormValues {
  name: string
}

export function FontDialog({
  open,
  onOpenChange,
  onRegister,
}: FontDialogProps) {
  const [error, setError] = useState('')
  const form = useForm<FontFormValues>({ defaultValues: { name: '' } })

  useEffect(() => {
    if (open) {
      form.reset({ name: '' })
      setError('')
    }
  }, [form, open])

  const submit = form.handleSubmit(async ({ name }) => {
    const normalized = name.trim()
    if (!normalized) {
      setError('请填写完整')
      return
    }
    setError('')
    try {
      await onRegister(normalized)
      onOpenChange(false)
    } catch (reason) {
      setError(readMessage(reason))
    }
  })

  return (
    <Dialog open={open} onOpenChange={onOpenChange} title="添加字体">
      <form className="settings-form" onSubmit={submit}>
        <label>
          字体名称
          <input
            aria-label="字体名称"
            placeholder="Enter name..."
            {...form.register('name')}
          />
        </label>
        <p>字体文件将在点击添加后由 Rust 原生文件对话框选择。</p>
        {error && <p role="alert">{error}</p>}
        <div className="dialog-actions">
          <button type="button" onClick={() => onOpenChange(false)}>
            取消
          </button>
          <button type="submit">添加</button>
        </div>
      </form>
    </Dialog>
  )
}

function readMessage(reason: unknown): string {
  return typeof reason === 'object' && reason !== null && 'message' in reason
    ? String(reason.message)
    : '字体添加失败'
}
