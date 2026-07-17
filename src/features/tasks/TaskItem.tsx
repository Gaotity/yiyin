import { useEffect, useState } from 'react'
import type { PlatformClient } from '../../platform/client'
import type { MetadataDto, TaskDescriptorDto } from '../../platform/types'

interface TaskItemProps {
  task: TaskDescriptorDto
  selected: boolean
  readExif: PlatformClient['readTaskExif']
  onSelect(id: string): void
  onError(error: unknown): void
}

const STATE_LABELS: Record<TaskDescriptorDto['state'], string> = {
  registered: '等待处理',
  queued: '等待处理',
  running: '处理中',
  completed: '输出完成',
  failed: '处理失败',
  cancelled: '已取消',
}

export function TaskItem({
  task,
  selected,
  readExif,
  onSelect,
  onError,
}: TaskItemProps) {
  const [metadata, setMetadata] = useState<MetadataDto | null>(null)
  const [metadataLoaded, setMetadataLoaded] = useState(false)

  useEffect(() => {
    let active = true
    setMetadataLoaded(false)
    void readExif(task.id)
      .then((value) => {
        if (active) {
          setMetadata(value)
          setMetadataLoaded(true)
        }
      })
      .catch((error: unknown) => {
        if (active) {
          setMetadata(null)
          setMetadataLoaded(true)
          onError(error)
        }
      })
    return () => {
      active = false
    }
  }, [onError, readExif, task.id])

  const copyMetadata = async () => {
    if (!metadata) {
      return
    }
    try {
      await navigator.clipboard.writeText(
        JSON.stringify(metadata.fields, null, 2),
      )
    } catch (error) {
      onError(error)
    }
  }
  const stateLabel =
    task.preview && task.state === 'completed'
      ? '预览完成'
      : STATE_LABELS[task.state]

  return (
    <article className="task-item">
      <button
        className={selected ? 'task selected' : 'task'}
        type="button"
        onClick={() => onSelect(task.id)}
      >
        <span>{task.displayName}</span>
        <span>{stateLabel}</span>
        <span>{task.progress}%</span>
      </button>
      <div className="task-metadata">
        <span>相机信息:</span>
        <span>
          {metadata
            ? Object.values(metadata.fields).join(' · ')
            : metadataLoaded
              ? '无'
              : '读取中...'}
        </span>
        {metadata && (
          <button
            type="button"
            aria-label={`复制 ${task.displayName} 相机信息`}
            onClick={() => void copyMetadata()}
          >
            复制
          </button>
        )}
      </div>
    </article>
  )
}
