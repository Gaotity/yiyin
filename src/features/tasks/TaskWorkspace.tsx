import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { parseCommandError, type PlatformClient } from '../../platform/client'
import type {
  CommandErrorDto,
  PublicConfigDto,
  TaskDescriptorDto,
} from '../../platform/types'
import { OutputControls } from './OutputControls'
import { PreviewPane } from './PreviewPane'
import { TaskList } from './TaskList'

interface TaskWorkspaceProps {
  config: PublicConfigDto
  tasks: TaskDescriptorDto[]
  selectedId: string | null
  dropError: CommandErrorDto | null
  readExif: PlatformClient['readTaskExif']
  onSelect(id: string): void
  onChooseImages(): Promise<void>
  onStartTasks(): Promise<void>
  onPreviewTask(id: string): Promise<void>
  onCancelTask(id: string): Promise<void>
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
  dropError,
  readExif,
  onSelect,
  onChooseImages,
  onStartTasks,
  onPreviewTask,
  onCancelTask,
  onInvalidatePreview,
  onChooseOutput,
  onOpenOutput,
  onClear,
  onOpenFields,
  onOpenTemplates,
}: TaskWorkspaceProps) {
  const [error, setError] = useState<string | null>(null)
  const previewTimer = useRef<number | null>(null)
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
  const clearPreviewTimer = useCallback(() => {
    if (previewTimer.current !== null) {
      window.clearTimeout(previewTimer.current)
      previewTimer.current = null
    }
  }, [])
  const previewEnabled = config.options.previewVisible
  const previewConfiguration = JSON.stringify(config)

  useEffect(() => {
    clearPreviewTimer()
    if (!previewEnabled || !selectedId) {
      onInvalidatePreview()
      return
    }
    void previewConfiguration
    previewTimer.current = window.setTimeout(() => {
      previewTimer.current = null
      void run(() => onPreviewTask(selectedId))
    }, 300)
    return clearPreviewTimer
  }, [
    clearPreviewTimer,
    onInvalidatePreview,
    onPreviewTask,
    previewConfiguration,
    previewEnabled,
    run,
    selectedId,
  ])

  const startOutput = useCallback(async () => {
    clearPreviewTimer()
    onInvalidatePreview()
    await run(onStartTasks)
  }, [clearPreviewTimer, onInvalidatePreview, onStartTasks, run])

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
        onCancel={(id) => void run(() => onCancelTask(id))}
        onError={showError}
      />
      <OutputControls
        tasks={tasks}
        onChooseOutput={() => void run(onChooseOutput)}
        onOpenOutput={() => void run(onOpenOutput)}
        onClear={() => void run(onClear)}
      />
      {(error ?? dropError?.message) && (
        <p className="task-error" role="alert">
          {error ?? dropError?.message}
        </p>
      )}
      <div className="primary-actions">
        <button type="button" onClick={() => void run(onChooseImages)}>
          添加图片
        </button>
        <button
          type="button"
          disabled={tasks.length === 0}
          onClick={() => void startOutput()}
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
