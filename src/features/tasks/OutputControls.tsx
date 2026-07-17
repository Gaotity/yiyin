import type { TaskDescriptorDto } from '../../platform/types'

interface OutputControlsProps {
  tasks: TaskDescriptorDto[]
  onChooseOutput(): void
  onOpenOutput(): void
  onClear(): void
}

export function OutputControls({
  tasks,
  onChooseOutput,
  onOpenOutput,
  onClear,
}: OutputControlsProps) {
  const completed = tasks.filter(
    (task) => !task.preview && task.state === 'completed',
  ).length

  return (
    <div className="output-controls">
      <span>
        {completed}/{tasks.length} 已完成
      </span>
      <button type="button" onClick={onChooseOutput}>
        选择输出目录
      </button>
      <button type="button" onClick={onOpenOutput}>
        打开输出目录
      </button>
      <button type="button" onClick={onClear} disabled={tasks.length === 0}>
        清空
      </button>
    </div>
  )
}
