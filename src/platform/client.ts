import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  BootstrapDto,
  CommandErrorDto,
  ExternalDestinationDto,
  MetadataDto,
  PublicConfigDto,
  ResourceDescriptorDto,
  TaskDescriptorDto,
  TaskStatusEventDto,
  UpdateConfigRequestDto,
} from './types'

export interface PlatformClient {
  bootstrap(): Promise<BootstrapDto>
  updateConfig(request: UpdateConfigRequestDto): Promise<PublicConfigDto>
  resetConfig(): Promise<PublicConfigDto>
  chooseOutputDirectory(): Promise<PublicConfigDto>
  openOutputDirectory(): Promise<void>
  chooseImages(): Promise<TaskDescriptorDto[]>
  registerFont(name: string): Promise<ResourceDescriptorDto>
  removeFont(id: string): Promise<ResourceDescriptorDto[]>
  registerOverlay(): Promise<ResourceDescriptorDto>
  readTaskExif(id: string): Promise<MetadataDto | null>
  startTasks(ids: string[]): Promise<TaskDescriptorDto[]>
  previewTask(id: string): Promise<TaskDescriptorDto[]>
  cancelTask(id: string): Promise<TaskDescriptorDto[]>
  clearTasks(): Promise<TaskDescriptorDto[]>
  minimizeWindow(): Promise<void>
  closeWindow(): Promise<void>
  openExternalUrl(destination: ExternalDestinationDto): Promise<void>
  onTaskStatus(
    listener: (event: TaskStatusEventDto) => void,
  ): Promise<() => void>
  onDropError(listener: (error: CommandErrorDto) => void): Promise<() => void>
}

const INTERNAL_ERROR: CommandErrorDto = {
  code: 'INTERNAL',
  message: 'An internal error occurred.',
}

export function parseCommandError(value: unknown): CommandErrorDto {
  if (typeof value === 'string') {
    try {
      return parseCommandError(JSON.parse(value))
    } catch {
      return INTERNAL_ERROR
    }
  }
  if (typeof value !== 'object' || value === null) {
    return INTERNAL_ERROR
  }
  const candidate = value as Record<string, unknown>
  if (
    typeof candidate.code !== 'string' ||
    typeof candidate.message !== 'string'
  ) {
    return INTERNAL_ERROR
  }
  return { code: candidate.code, message: candidate.message }
}

async function invokeCommand<T>(command: string, request?: object): Promise<T> {
  try {
    return request === undefined
      ? await invoke<T>(command)
      : await invoke<T>(command, { request })
  } catch (error) {
    throw parseCommandError(error)
  }
}

export const tauriClient: PlatformClient = {
  bootstrap: () => invokeCommand('bootstrap'),
  updateConfig: (request) => invokeCommand('update_config', request),
  resetConfig: () => invokeCommand('reset_config'),
  chooseOutputDirectory: () => invokeCommand('choose_output_directory'),
  openOutputDirectory: () => invokeCommand('open_output_directory'),
  chooseImages: () => invokeCommand('choose_images'),
  registerFont: (name) => invokeCommand('register_font', { name }),
  removeFont: (id) => invokeCommand('remove_font', { id }),
  registerOverlay: () => invokeCommand('register_overlay'),
  readTaskExif: (id) => invokeCommand('read_task_exif', { id }),
  startTasks: (ids) => invokeCommand('start_tasks', { ids }),
  previewTask: (id) => invokeCommand('preview_task', { id }),
  cancelTask: (id) => invokeCommand('cancel_task', { id }),
  clearTasks: () => invokeCommand('clear_tasks'),
  minimizeWindow: () => invokeCommand('minimize_window'),
  closeWindow: () => invokeCommand('close_window'),
  openExternalUrl: (destination) =>
    invokeCommand('open_external_url', { destination }),
  async onTaskStatus(listener) {
    try {
      return await listen<TaskStatusEventDto>('task-status', ({ payload }) =>
        listener(payload),
      )
    } catch (error) {
      throw parseCommandError(error)
    }
  },
  async onDropError(listener) {
    try {
      return await listen<CommandErrorDto>('drop-error', ({ payload }) =>
        listener(payload),
      )
    } catch (error) {
      throw parseCommandError(error)
    }
  },
}
