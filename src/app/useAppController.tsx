import { useCallback, useEffect, useReducer } from 'react'
import { parseCommandError, type PlatformClient } from '../platform/client'
import type {
  BootstrapDto,
  CommandErrorDto,
  PublicConfigDto,
  ResourceDescriptorDto,
  TaskDescriptorDto,
  TaskStatusEventDto,
} from '../platform/types'

type Phase = 'loading' | 'ready' | 'error'

interface AppState {
  phase: Phase
  snapshot: BootstrapDto | null
  error: CommandErrorDto | null
  selectedTaskId: string | null
  pendingEvents: Record<string, TaskStatusEventDto>
  refreshRequest: number
}

type AppAction =
  | { type: 'bootstrapped'; snapshot: BootstrapDto }
  | { type: 'bootstrap-failed'; error: CommandErrorDto }
  | { type: 'task-status'; event: TaskStatusEventDto }
  | { type: 'tasks-replaced'; tasks: TaskDescriptorDto[] }
  | { type: 'config-replaced'; config: PublicConfigDto }
  | { type: 'resource-upserted'; resource: ResourceDescriptorDto }
  | { type: 'fonts-replaced'; fonts: ResourceDescriptorDto[] }
  | { type: 'selected'; id: string | null }

const initialState: AppState = {
  phase: 'loading',
  snapshot: null,
  error: null,
  selectedTaskId: null,
  pendingEvents: {},
  refreshRequest: 0,
}

export function useAppController(client: PlatformClient) {
  const [state, dispatch] = useReducer(reducer, initialState)

  const load = useCallback(async () => {
    try {
      dispatch({ type: 'bootstrapped', snapshot: await client.bootstrap() })
    } catch (error) {
      dispatch({ type: 'bootstrap-failed', error: parseCommandError(error) })
    }
  }, [client])

  useEffect(() => {
    let active = true
    let unlisten: (() => void) | undefined
    void client
      .onTaskStatus((event) => {
        if (active) {
          dispatch({ type: 'task-status', event })
        }
      })
      .then((cleanup) => {
        if (active) {
          unlisten = cleanup
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
    void load()
    return () => {
      active = false
      unlisten?.()
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

  return {
    ...state,
    retry: load,
    selectTask: (id: string | null) => dispatch({ type: 'selected', id }),
    chooseImages: async () => replaceTasks(await client.chooseImages()),
    startTasks: async () => {
      const ids = state.snapshot?.tasks.map((task) => task.id) ?? []
      replaceTasks(await client.startTasks(ids))
    },
    clearTasks: async () => replaceTasks(await client.clearTasks()),
    resetConfig: async () => replaceConfig(await client.resetConfig()),
    updateConfig: async (config: PublicConfigDto) => {
      const canonical = await client.updateConfig({ config })
      replaceConfig(canonical)
      return canonical
    },
    registerFont: async (name: string) => {
      const resource = await client.registerFont(name)
      dispatch({ type: 'resource-upserted', resource })
      return resource
    },
    removeFont: async (id: string) => {
      const fonts = await client.removeFont(id)
      dispatch({ type: 'fonts-replaced', fonts })
      return fonts
    },
    registerOverlay: async () => {
      const resource = await client.registerOverlay()
      dispatch({ type: 'resource-upserted', resource })
      return resource
    },
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
