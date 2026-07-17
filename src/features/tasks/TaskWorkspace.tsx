import { useCallback, useEffect, useMemo, useState } from 'react'
import { parseCommandError, type PlatformClient } from '../../platform/client'
import type { PublicConfigDto, TaskDescriptorDto } from '../../platform/types'
import { OutputControls } from './OutputControls'
import { PreviewPane } from './PreviewPane'
import { TaskList } from './TaskList'

interface TaskWorkspaceProps {
  config: PublicConfigDto
  tasks: TaskDescriptorDto[]
  selectedId: string | null
  readExif: PlatformClient['readTaskExif']
  onSelect(id: string): void
  onChooseImages(): Promise<void>
  onStartTasks(): Promise<void>
  onPreviewTask(id: string): Promise<void>
  onInvalidatePreview(): void
  onChooseOutput(): Promise<void>
  onOpenOutput(): Promise<void>
  onClear(): Promise<void>
  onOpenFields(): void
  onOpenTemplates(): void
}

export function TaskWorkspace({
  config,
  tasks,
  selectedId,
  readExif,
  onSelect,
  onChooseImages,
  onStartTasks,
  onPreviewTask,
  onInvalidatePreview,
  onChooseOutput,
  onOpenOutput,
  onClear,
  onOpenFields,
  onOpenTemplates,
}: TaskWorkspaceProps) {
  const [error, setError] = useState<string | null>(null)
  const selectedTask = useMemo(
    () => tasks.find((task) => task.id === selectedId) ?? null,
    [selectedId, tasks],
  )
  const showError = useCallback((value: unknown) => {
    setError(parseCommandError(value).message)
  }, [])
  const run = useCallback(
    async (operation: () => Promise<void>) => {
      setError(null)
      try {
        await operation()
      } catch (value) {
        showError(value)
      }
    },
    [showError],
  )

  useEffect(() => {
    if (!config.options.previewVisible || !selectedId) {
      onInvalidatePreview()
      return
    }
    const timer = window.setTimeout(() => {
      void run(() => onPreviewTask(selectedId))
    }, 300)
    return () => window.clearTimeout(timer)
  }, [config, onInvalidatePreview, onPreviewTask, run, selectedId])

  return (
    <div className="task-workspace">
      <PreviewPane
        enabled={config.options.previewVisible}
        task={selectedTask}
      />
      <TaskList
        tasks={tasks}
        selectedId={selectedId}
        readExif={readExif}
        onSelect={onSelect}
        onError={showError}
      />
      <OutputControls
        tasks={tasks}
        onChooseOutput={() => void run(onChooseOutput)}
        onOpenOutput={() => void run(onOpenOutput)}
        onClear={() => void run(onClear)}
      />
      {error && (
        <p className="task-error" role="alert">
          {error}
        </p>
      )}
      <div className="primary-actions">
        <button type="button" onClick={() => void run(onChooseImages)}>
          添加图片
        </button>
        <button
          type="button"
          disabled={tasks.length === 0}
          onClick={() => void run(onStartTasks)}
        >
          生成印框
        </button>
        <button type="button" onClick={onOpenFields}>
          参数设置
        </button>
        <button type="button" onClick={onOpenTemplates}>
          模板设置
        </button>
      </div>
    </div>
  )
}
