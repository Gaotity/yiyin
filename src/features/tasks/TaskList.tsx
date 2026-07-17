import type { PlatformClient } from '../../platform/client'
import type { TaskDescriptorDto } from '../../platform/types'
import { TaskItem } from './TaskItem'

interface TaskListProps {
  tasks: TaskDescriptorDto[]
  selectedId: string | null
  readExif: PlatformClient['readTaskExif']
  onSelect(id: string): void
  onError(error: unknown): void
}

export function TaskList({
  tasks,
  selectedId,
  readExif,
  onSelect,
  onError,
}: TaskListProps) {
  return (
    <section className="task-list" aria-label="图片任务">
      {tasks.length === 0 && <p className="task-empty">请添加图片</p>}
      {tasks.map((task) => (
        <TaskItem
          key={task.id}
          task={task}
          selected={task.id === selectedId}
          readExif={readExif}
          onSelect={onSelect}
          onError={onError}
        />
      ))}
    </section>
  )
}
