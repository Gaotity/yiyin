import { useCallback, useEffect, useMemo, useReducer, useRef } from 'react'
import { parseCommandError, type PlatformClient } from '../platform/client'
import type {
  BootstrapDto,
  CommandErrorDto,
  PublicConfigDto,
  ResourceDescriptorDto,
  TaskDescriptorDto,
  TaskStatusEventDto,
} from '../platform/types'

const PREVIEW_DEBOUNCE_MS = 300

type Phase = 'loading' | 'ready' | 'error'

interface AppState {
  phase: Phase
  snapshot: BootstrapDto | null
  error: CommandErrorDto | null
  dropError: CommandErrorDto | null
  selectedTaskId: string | null
  pendingEvents: Record<string, TaskStatusEventDto>
  refreshRequest: number
}

type AppAction =
  | { type: 'bootstrapped'; snapshot: BootstrapDto }
  | { type: 'bootstrap-failed'; error: CommandErrorDto }
  | { type: 'task-status'; event: TaskStatusEventDto }
  | { type: 'drop-error'; error: CommandErrorDto }
  | { type: 'tasks-replaced'; tasks: TaskDescriptorDto[] }
  | { type: 'config-replaced'; config: PublicConfigDto }
  | { type: 'resource-upserted'; resource: ResourceDescriptorDto }
  | { type: 'fonts-replaced'; fonts: ResourceDescriptorDto[] }
  | { type: 'selected'; id: string | null }

const initialState: AppState = {
  phase: 'loading',
  snapshot: null,
  error: null,
  dropError: null,
  selectedTaskId: null,
  pendingEvents: {},
  refreshRequest: 0,
}

export function useAppController(client: PlatformClient) {
  const [state, dispatch] = useReducer(reducer, initialState)
  const previewRequest = useRef(0)
  const previewTimer = useRef<number | null>(null)

  const load = useCallback(async () => {
    try {
      dispatch({ type: 'bootstrapped', snapshot: await client.bootstrap() })
    } catch (error) {
      dispatch({ type: 'bootstrap-failed', error: parseCommandError(error) })
    }
  }, [client])

  useEffect(() => {
    let active = true
    const cleanups: Array<() => void> = []
    const subscribe = (listen: () => Promise<() => void>) => {
      void listen()
        .then((cleanup) => {
          if (active) {
            cleanups.push(cleanup)
          } else {
            cleanup()
          }
        })
        .catch((error: unknown) => {
          if (active) {
            dispatch({
              type: 'bootstrap-failed',
              error: parseCommandError(error),
            })
          }
        })
    }
    subscribe(() =>
      client.onTaskStatus((event) => {
        if (active) {
          dispatch({ type: 'task-status', event })
        }
      }),
    )
    subscribe(() =>
      client.onDropError((error) => {
        if (active) {
          dispatch({ type: 'drop-error', error })
        }
      }),
    )
    void load()
    return () => {
      active = false
      for (const cleanup of cleanups) {
        cleanup()
      }
    }
  }, [client, load])

  useEffect(() => {
    if (state.refreshRequest > 0) {
      void load()
    }
  }, [load, state.refreshRequest])

  const replaceTasks = useCallback((tasks: TaskDescriptorDto[]) => {
    dispatch({ type: 'tasks-replaced', tasks })
  }, [])

  const replaceConfig = useCallback((config: PublicConfigDto) => {
    dispatch({ type: 'config-replaced', config })
  }, [])

  const selectTask = useCallback((id: string | null) => {
    dispatch({ type: 'selected', id })
  }, [])

  const chooseImages = useCallback(async () => {
    replaceTasks(await client.chooseImages())
  }, [client, replaceTasks])

  const previewTask = useCallback(
    async (id: string) => {
      const request = ++previewRequest.current
      try {
        const tasks = await client.previewTask(id)
        if (request === previewRequest.current) {
          replaceTasks(tasks)
        }
      } catch (error) {
        if (request === previewRequest.current) {
          throw error
        }
      }
    },
    [client, replaceTasks],
  )

  const invalidatePreviewRequests = useCallback(() => {
    previewRequest.current += 1
  }, [])

  const clearPreviewTimer = useCallback(() => {
    if (previewTimer.current !== null) {
      window.clearTimeout(previewTimer.current)
      previewTimer.current = null
    }
  }, [])

  const startTasks = useCallback(async () => {
    clearPreviewTimer()
    invalidatePreviewRequests()
    const tasks = state.snapshot?.tasks ?? []
    const selected = state.selectedTaskId
    const ids = selected
      ? [
          selected,
          ...tasks.filter((task) => task.id !== selected).map(({ id }) => id),
        ]
      : tasks.map(({ id }) => id)
    replaceTasks(await client.startTasks(ids))
  }, [
    clearPreviewTimer,
    client,
    invalidatePreviewRequests,
    replaceTasks,
    state.selectedTaskId,
    state.snapshot?.tasks,
  ])

  const clearTasks = useCallback(async () => {
    invalidatePreviewRequests()
    replaceTasks(await client.clearTasks())
  }, [client, invalidatePreviewRequests, replaceTasks])

  const previewEnabled = state.snapshot?.config.options.previewVisible
  // Detect config changes by value rather than by reference: an equivalent
  // bootstrap refresh yields a new config object with identical content and
  // must not re-trigger a preview, so reference equality would misfire here.
  const configSignature = useMemo(
    () => JSON.stringify(state.snapshot?.config),
    [state.snapshot?.config],
  )

  useEffect(() => {
    const selectedTaskId = state.selectedTaskId
    if (!previewEnabled || !selectedTaskId) {
      invalidatePreviewRequests()
      return
    }
    void configSignature
    previewTimer.current = window.setTimeout(() => {
      previewTimer.current = null
      // previewTask swallows stale rejections internally; a rejection that
      // reaches this catch is current and must surface in the task banner.
      previewTask(selectedTaskId).catch((error: unknown) => {
        dispatch({ type: 'drop-error', error: parseCommandError(error) })
      })
    }, PREVIEW_DEBOUNCE_MS)
    return clearPreviewTimer
  }, [
    clearPreviewTimer,
    configSignature,
    invalidatePreviewRequests,
    previewEnabled,
    previewTask,
    state.selectedTaskId,
  ])

  const cancelTask = useCallback(
    async (id: string) => {
      replaceTasks(await client.cancelTask(id))
    },
    [client, replaceTasks],
  )

  const chooseOutputDirectory = useCallback(async () => {
    replaceConfig(await client.chooseOutputDirectory())
  }, [client, replaceConfig])

  const openOutputDirectory = useCallback(async () => {
    await client.openOutputDirectory()
  }, [client])

  const readTaskExif = useCallback(
    (id: string) => client.readTaskExif(id),
    [client],
  )

  const resetConfig = useCallback(async () => {
    replaceConfig(await client.resetConfig())
  }, [client, replaceConfig])

  const updateConfig = useCallback(
    async (config: PublicConfigDto) => {
      const canonical = await client.updateConfig({ config })
      replaceConfig(canonical)
      return canonical
    },
    [client, replaceConfig],
  )

  const registerFont = useCallback(
    async (name: string) => {
      const resource = await client.registerFont(name)
      dispatch({ type: 'resource-upserted', resource })
      return resource
    },
    [client],
  )

  const removeFont = useCallback(
    async (id: string) => {
      const fonts = await client.removeFont(id)
      dispatch({ type: 'fonts-replaced', fonts })
      return fonts
    },
    [client],
  )

  const registerOverlay = useCallback(async () => {
    const resource = await client.registerOverlay()
    dispatch({ type: 'resource-upserted', resource })
    return resource
  }, [client])

  return {
    ...state,
    retry: load,
    selectTask,
    chooseImages,
    startTasks,
    clearTasks,
    cancelTask,
    chooseOutputDirectory,
    openOutputDirectory,
    readTaskExif,
    resetConfig,
    updateConfig,
    registerFont,
    removeFont,
    registerOverlay,
  }
}

function reducer(state: AppState, action: AppAction): AppState {
  switch (action.type) {
    case 'bootstrapped': {
      const tasks = action.snapshot.tasks.map((task) => {
        const pending = state.pendingEvents[task.id]
        return pending ? applyStatus(task, pending) : task
      })
      const snapshot = { ...action.snapshot, tasks }
      return {
        ...state,
        phase: 'ready',
        snapshot,
        error: null,
        selectedTaskId: selectExisting(state.selectedTaskId, tasks),
        pendingEvents: {},
      }
    }
    case 'bootstrap-failed':
      return { ...state, phase: 'error', error: action.error }
    case 'drop-error':
      return { ...state, dropError: action.error }
    case 'task-status': {
      if (!state.snapshot) {
        return {
          ...state,
          pendingEvents: {
            ...state.pendingEvents,
            [action.event.taskId]: action.event,
          },
        }
      }
      const known = state.snapshot.tasks.some(
        (task) => task.id === action.event.taskId,
      )
      if (!known) {
        return { ...state, refreshRequest: state.refreshRequest + 1 }
      }
      const tasks = state.snapshot.tasks.map((task) =>
        task.id === action.event.taskId
          ? applyStatus(task, action.event)
          : task,
      )
      return {
        ...state,
        snapshot: { ...state.snapshot, tasks },
        dropError: null,
        refreshRequest: isTerminal(action.event)
          ? state.refreshRequest + 1
          : state.refreshRequest,
      }
    }
    case 'tasks-replaced':
      return state.snapshot
        ? {
            ...state,
            snapshot: { ...state.snapshot, tasks: action.tasks },
            dropError: null,
            selectedTaskId: selectExisting(state.selectedTaskId, action.tasks),
          }
        : state
    case 'config-replaced':
      return state.snapshot
        ? { ...state, snapshot: { ...state.snapshot, config: action.config } }
        : state
    case 'resource-upserted':
      return state.snapshot
        ? {
            ...state,
            snapshot: {
              ...state.snapshot,
              resources: [
                ...state.snapshot.resources.filter(
                  (resource) => resource.id !== action.resource.id,
                ),
                action.resource,
              ],
            },
          }
        : state
    case 'fonts-replaced':
      return state.snapshot
        ? {
            ...state,
            snapshot: {
              ...state.snapshot,
              resources: [
                ...state.snapshot.resources.filter(
                  (resource) => resource.kind !== 'font',
                ),
                ...action.fonts,
              ],
            },
          }
        : state
    case 'selected':
      return {
        ...state,
        selectedTaskId:
          action.id &&
          state.snapshot?.tasks.some((task) => task.id === action.id)
            ? action.id
            : null,
      }
  }
}

function applyStatus(
  task: TaskDescriptorDto,
  event: TaskStatusEventDto,
): TaskDescriptorDto {
  return {
    ...task,
    state: event.state,
    progress: event.progress,
    preview: event.preview,
  }
}

function selectExisting(
  current: string | null,
  tasks: TaskDescriptorDto[],
): string | null {
  return tasks.some((task) => task.id === current)
    ? current
    : (tasks[0]?.id ?? null)
}

function isTerminal(event: TaskStatusEventDto): boolean {
  return ['completed', 'failed', 'cancelled'].includes(event.state)
}
