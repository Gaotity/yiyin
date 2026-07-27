import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { TaskStatusEventDto } from './types'

const { invoke, listen } = vi.hoisted(() => ({
  invoke: vi.fn(),
  listen: vi.fn(),
}))

vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen }))

import { parseCommandError, tauriClient } from './client'

describe('tauriClient', () => {
  beforeEach(() => {
    invoke.mockReset()
    listen.mockReset()
  })

  it('uses the exact dedicated command names and request payloads', async () => {
    invoke.mockResolvedValue(undefined)

    await tauriClient.bootstrap()
    await tauriClient.updateConfig({ config: {} as never })
    await tauriClient.resetConfig()
    await tauriClient.chooseOutputDirectory()
    await tauriClient.openOutputDirectory()
    await tauriClient.chooseImages()
    await tauriClient.registerFont('Body')
    await tauriClient.removeFont('font-1')
    await tauriClient.registerOverlay()
    await tauriClient.readTaskExif('task-1')
    await tauriClient.startTasks(['task-1'])
    await tauriClient.previewTask('task-1')
    await tauriClient.cancelTask('task-1')
    await tauriClient.clearTasks()
    await tauriClient.minimizeWindow()
    await tauriClient.closeWindow()
    await tauriClient.openExternalUrl('issues')

    expect(invoke.mock.calls).toEqual([
      ['bootstrap'],
      ['update_config', { request: { config: {} } }],
      ['reset_config'],
      ['choose_output_directory'],
      ['open_output_directory'],
      ['choose_images'],
      ['register_font', { request: { name: 'Body' } }],
      ['remove_font', { request: { id: 'font-1' } }],
      ['register_overlay'],
      ['read_task_exif', { request: { id: 'task-1' } }],
      ['start_tasks', { request: { ids: ['task-1'] } }],
      ['preview_task', { request: { id: 'task-1' } }],
      ['cancel_task', { request: { id: 'task-1' } }],
      ['clear_tasks'],
      ['minimize_window'],
      ['close_window'],
      ['open_external_url', { request: { destination: 'issues' } }],
    ])
  })

  it('forwards task payloads and returns the exact unlisten callback', async () => {
    const unlisten = vi.fn()
    let dispatch: ((event: { payload: TaskStatusEventDto }) => void) | undefined
    listen.mockImplementation(async (_name, handler) => {
      dispatch = handler
      return unlisten
    })
    const listener = vi.fn()
    const cleanup = await tauriClient.onTaskStatus(listener)
    const event: TaskStatusEventDto = {
      taskId: 'task-1',
      state: 'running',
      progress: 30,
      preview: false,
      cancellationReason: null,
    }

    dispatch?.({ payload: event })

    expect(listen).toHaveBeenCalledWith('task-status', expect.any(Function))
    expect(listener).toHaveBeenCalledWith(event)
    expect(cleanup).toBe(unlisten)
  })

  it('parses only safe command errors and redacts unknown rejection values', async () => {
    expect(
      parseCommandError({ code: 'FILE_INVALID', message: 'Invalid image.' }),
    ).toEqual({
      code: 'FILE_INVALID',
      message: 'Invalid image.',
    })
    expect(
      parseCommandError('{"code":"TASK_NOT_FOUND","message":"Missing task."}'),
    ).toEqual({
      code: 'TASK_NOT_FOUND',
      message: 'Missing task.',
    })
    expect(parseCommandError('/Users/private/secret.jpg')).toEqual({
      code: 'INTERNAL',
      message: 'An internal error occurred.',
    })

    invoke.mockRejectedValue({ code: 'FORBIDDEN', message: 'Not allowed.' })
    await expect(tauriClient.bootstrap()).rejects.toEqual({
      code: 'FORBIDDEN',
      message: 'Not allowed.',
    })
  })
})
