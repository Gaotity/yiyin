import type { TaskDescriptorDto } from '../../platform/types'

interface PreviewPaneProps {
  enabled: boolean
  task: TaskDescriptorDto | null
}

export function PreviewPane({ enabled, task }: PreviewPaneProps) {
  if (!enabled || !task) {
    return null
  }

  if (task.preview && task.state === 'failed') {
    return (
      <section className="preview-pane preview-error" aria-label="图片预览">
        预览图生成失败
      </section>
    )
  }

  if (
    task.state === 'completed' &&
    ((task.preview && task.resource?.kind === 'preview') ||
      (!task.preview && task.resource?.kind === 'output'))
  ) {
    return (
      <section className="preview-pane" aria-label="图片预览">
        <img src={task.resource.url} alt="预览图" />
      </section>
    )
  }

  return (
    <section className="preview-pane preview-loading" aria-label="图片预览">
      生成预览中...
    </section>
  )
}
