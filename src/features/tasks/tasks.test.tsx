import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { App } from '../../app/App'
import { createFakePlatformClient, defaultBootstrap } from '../../platform/fake'
import type { TaskDescriptorDto } from '../../platform/types'
import { PreviewPane } from './PreviewPane'

afterEach(() => {
  cleanup()
  vi.useRealTimers()
})

function task(
  id: string,
  displayName: string,
  overrides: Partial<TaskDescriptorDto> = {},
): TaskDescriptorDto {
  return {
    id,
    displayName,
    state: 'registered',
    progress: 0,
    preview: false,
    resource: null,
    ...overrides,
  }
}

describe('task workflows', () => {
  it('adds images natively, selects the first task, and starts the selected task first', async () => {
    const fake = createFakePlatformClient()
    const added = [task('task-one', 'one.jpg'), task('task-two', 'two.jpg')]
    const chooseImages = vi
      .spyOn(fake.client, 'chooseImages')
      .mockResolvedValue(added)
    const startTasks = vi
      .spyOn(fake.client, 'startTasks')
      .mockResolvedValue(added)
    render(<App client={fake.client} />)

    fireEvent.click(await screen.findByRole('button', { name: '添加图片' }))
    expect(chooseImages).toHaveBeenCalledTimes(1)
    expect(
      (await screen.findByRole('button', { name: /one\.jpg/ })).className,
    ).toContain('selected')

    fireEvent.click(screen.getByRole('button', { name: /two\.jpg/ }))
    fireEvent.click(screen.getByRole('button', { name: '生成印框' }))

    await waitFor(() =>
      expect(startTasks).toHaveBeenCalledWith(['task-two', 'task-one']),
    )
  })

  it('refreshes native drag/drop registration and does not double-start quick output', async () => {
    const quickConfig = defaultBootstrap().config
    quickConfig.options.quickOutput = true
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ config: quickConfig }),
    })
    const dropped = task('task-drop', 'dropped.jpg', { state: 'queued' })
    const startTasks = vi.spyOn(fake.client, 'startTasks')
    vi.spyOn(fake.client, 'chooseImages').mockResolvedValue([dropped])
    render(<App client={fake.client} />)
    await screen.findByRole('button', { name: '添加图片' })

    fake.setSnapshot(
      defaultBootstrap({ config: quickConfig, tasks: [dropped] }),
    )
    act(() => {
      fake.emitTaskStatus({
        taskId: dropped.id,
        state: 'queued',
        progress: 0,
        preview: false,
        cancellationReason: null,
      })
    })
    expect(await screen.findByText('dropped.jpg')).toBeTruthy()

    fireEvent.click(screen.getByRole('button', { name: '添加图片' }))
    await waitFor(() => expect(screen.getByText('等待处理')).toBeTruthy())
    expect(startTasks).not.toHaveBeenCalled()
  })

  it('surfaces native drag-and-drop registration failures in the error area', async () => {
    const fake = createFakePlatformClient()
    render(<App client={fake.client} />)
    await screen.findByRole('button', { name: '添加图片' })

    act(() => {
      fake.emitDropError({
        code: 'FILE_INVALID',
        message: 'The selected file is invalid.',
      })
    })

    const alert = await screen.findByRole('alert')
    expect(alert.textContent).toContain('The selected file is invalid.')
    expect(document.body.textContent).not.toContain('/Users/')
  })

  it('clears a stale drop error once task activity resumes', async () => {
    const existing = task('task-one', 'one.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [existing] }),
    })
    render(<App client={fake.client} />)
    await screen.findByRole('button', { name: '添加图片' })

    act(() => {
      fake.emitDropError({
        code: 'FILE_INVALID',
        message: 'The selected file is invalid.',
      })
    })
    expect(await screen.findByRole('alert')).toBeTruthy()

    act(() => {
      fake.emitTaskStatus({
        taskId: 'task-one',
        state: 'queued',
        progress: 0,
        preview: false,
        cancellationReason: null,
      })
    })

    await waitFor(() => expect(screen.queryByRole('alert')).toBeNull())
  })

  it('cancels a running task from the task list', async () => {
    const running = task('task-one', 'one.jpg', {
      state: 'running',
      progress: 40,
    })
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [running] }),
    })
    const cancelTask = vi.spyOn(fake.client, 'cancelTask')
    render(<App client={fake.client} />)

    fireEvent.click(await screen.findByRole('button', { name: '取消 one.jpg' }))

    await waitFor(() => expect(cancelTask).toHaveBeenCalledWith('task-one'))
    expect(await screen.findByText('已取消')).toBeTruthy()
  })

  it('shows progress, terminal states, loads EXIF, and copies normalized fields only', async () => {
    const first = task('task-one', 'one.jpg')
    const second = task('task-two', 'two.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [first, second] }),
    })
    vi.spyOn(fake.client, 'readTaskExif').mockImplementation((id) =>
      Promise.resolve(id === first.id ? { fields: { Model: 'A7R V' } } : null),
    )
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    })
    render(<App client={fake.client} />)

    expect((await screen.findAllByText('相机信息:')).length).toBe(2)
    fireEvent.click(
      await screen.findByRole('button', { name: '复制 one.jpg 相机信息' }),
    )
    expect(writeText).toHaveBeenCalledWith(
      JSON.stringify({ Model: 'A7R V' }, null, 2),
    )

    act(() => {
      fake.emitTaskStatus({
        taskId: first.id,
        state: 'running',
        progress: 90,
        preview: false,
        cancellationReason: null,
      })
    })
    expect(await screen.findByText('90%')).toBeTruthy()

    fake.setSnapshot(
      defaultBootstrap({
        tasks: [
          task(first.id, first.displayName, {
            state: 'completed',
            progress: 100,
            resource: {
              id: 'output-one',
              kind: 'output',
              displayName: 'one.jpg',
              url: 'yiyin://resource/output-one',
            },
          }),
          task(second.id, second.displayName, { state: 'failed' }),
        ],
      }),
    )
    act(() => {
      fake.emitTaskStatus({
        taskId: first.id,
        state: 'completed',
        progress: 100,
        preview: false,
        cancellationReason: null,
      })
    })
    expect(await screen.findByText('输出完成')).toBeTruthy()
    expect(screen.getByText('处理失败')).toBeTruthy()
  })

  it('clears tasks, owns output directory actions, and displays safe command errors', async () => {
    const existing = task('task-one', 'one.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [existing] }),
    })
    const clearTasks = vi.spyOn(fake.client, 'clearTasks').mockResolvedValue([])
    const chooseOutputDirectory = vi
      .spyOn(fake.client, 'chooseOutputDirectory')
      .mockImplementation(async () => defaultBootstrap().config)
    const openOutputDirectory = vi.spyOn(fake.client, 'openOutputDirectory')
    vi.spyOn(fake.client, 'chooseImages').mockRejectedValue({
      code: 'FILE_INVALID',
      message: 'The selected file is invalid.',
    })
    render(<App client={fake.client} />)
    await screen.findByText('one.jpg')

    fireEvent.click(screen.getByRole('button', { name: '选择输出目录' }))
    fireEvent.click(screen.getByRole('button', { name: '打开输出目录' }))
    fireEvent.click(screen.getByRole('button', { name: '清空' }))
    await waitFor(() => {
      expect(chooseOutputDirectory).toHaveBeenCalledTimes(1)
      expect(openOutputDirectory).toHaveBeenCalledTimes(1)
      expect(clearTasks).toHaveBeenCalledTimes(1)
    })
    expect(screen.queryByText('one.jpg')).toBeNull()

    fireEvent.click(screen.getByRole('button', { name: '添加图片' }))
    expect(
      await screen.findByText('The selected file is invalid.'),
    ).toBeTruthy()
    expect(document.body.textContent).not.toContain('/Users/')
  })
})

describe('preview reconciliation', () => {
  it('shows a completed export in the preview pane', () => {
    const completed = task('task-one', 'one.jpg', {
      state: 'completed',
      progress: 100,
      resource: {
        id: 'output-one',
        kind: 'output',
        displayName: 'one.jpg',
        url: 'yiyin://resource/output-one',
      },
    })

    render(<PreviewPane enabled task={completed} />)

    expect(
      screen.getByRole('img', { name: '预览图' }).getAttribute('src'),
    ).toBe('yiyin://resource/output-one')
  })

  it('cancels a pending preview when export starts', async () => {
    const existing = task('task-one', 'one.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [existing] }),
    })
    const previewTask = vi.spyOn(fake.client, 'previewTask')
    render(<App client={fake.client} />)
    await screen.findByRole('switch', { name: '实时预览' })
    vi.useFakeTimers()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    fireEvent.click(screen.getByRole('button', { name: '生成印框' }))
    await vi.advanceTimersByTimeAsync(300)

    expect(previewTask).not.toHaveBeenCalled()
  })

  it('ignores an in-flight preview failure after export starts', async () => {
    const existing = task('task-one', 'one.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [existing] }),
    })
    let rejectPreview: ((reason: unknown) => void) | undefined
    const previewTask = vi.spyOn(fake.client, 'previewTask').mockImplementation(
      () =>
        new Promise<TaskDescriptorDto[]>((_resolve, reject) => {
          rejectPreview = reject
        }),
    )
    render(<App client={fake.client} />)
    await screen.findByRole('switch', { name: '实时预览' })
    vi.useFakeTimers()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    await vi.advanceTimersByTimeAsync(300)
    expect(previewTask).toHaveBeenCalledTimes(1)
    fireEvent.click(screen.getByRole('button', { name: '生成印框' }))
    await act(async () => {
      rejectPreview?.({
        code: 'INVALID_REQUEST',
        message: 'The task is already running.',
      })
      await Promise.resolve()
    })

    expect(screen.queryByText('The task is already running.')).toBeNull()
  })

  it('does not regenerate a preview after an equivalent bootstrap refresh', async () => {
    const existing = task('task-one', 'one.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [existing] }),
    })
    const previewTask = vi
      .spyOn(fake.client, 'previewTask')
      .mockResolvedValue([existing])
    render(<App client={fake.client} />)
    await screen.findByRole('switch', { name: '实时预览' })
    vi.useFakeTimers()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    await vi.advanceTimersByTimeAsync(300)
    expect(previewTask).toHaveBeenCalledTimes(1)

    await act(async () => {
      fake.emitTaskStatus({
        taskId: existing.id,
        state: 'completed',
        progress: 100,
        preview: true,
        cancellationReason: null,
      })
      await Promise.resolve()
      await Promise.resolve()
    })
    expect(fake.calls.bootstrap).toBeGreaterThan(1)
    await vi.advanceTimersByTimeAsync(300)

    expect(previewTask).toHaveBeenCalledTimes(1)
  })

  it('debounces requests, ignores stale completion, clears when disabled, and shows current failure', async () => {
    const first = task('task-one', 'one.jpg')
    const second = task('task-two', 'two.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [first, second] }),
    })
    const pending: Array<{
      id: string
      resolve(tasks: TaskDescriptorDto[]): void
    }> = []
    const previewTask = vi.spyOn(fake.client, 'previewTask').mockImplementation(
      (id) =>
        new Promise<TaskDescriptorDto[]>((resolve) => {
          pending.push({ id, resolve })
        }),
    )
    render(<App client={fake.client} />)
    await screen.findByRole('switch', { name: '实时预览' })
    vi.useFakeTimers()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    await vi.advanceTimersByTimeAsync(299)
    expect(previewTask).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(1)
    expect(previewTask).toHaveBeenLastCalledWith(first.id)

    fireEvent.click(screen.getByRole('button', { name: /two\.jpg/ }))
    await vi.advanceTimersByTimeAsync(300)
    expect(previewTask).toHaveBeenLastCalledWith(second.id)

    const previewB = task(second.id, second.displayName, {
      state: 'completed',
      progress: 100,
      preview: true,
      resource: {
        id: 'preview-b',
        kind: 'preview',
        displayName: 'two.jpg',
        url: 'yiyin://resource/preview-b',
      },
    })
    await act(async () => {
      pending.find(({ id }) => id === second.id)?.resolve([first, previewB])
      await Promise.resolve()
    })
    expect(
      screen.getByRole('img', { name: '预览图' }).getAttribute('src'),
    ).toBe('yiyin://resource/preview-b')

    const previewA = task(first.id, first.displayName, {
      state: 'completed',
      progress: 100,
      preview: true,
      resource: {
        id: 'preview-a',
        kind: 'preview',
        displayName: 'one.jpg',
        url: 'yiyin://resource/preview-a',
      },
    })
    await act(async () => {
      pending.find(({ id }) => id === first.id)?.resolve([previewA, second])
      await Promise.resolve()
    })
    expect(
      screen.getByRole('img', { name: '预览图' }).getAttribute('src'),
    ).toBe('yiyin://resource/preview-b')

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    expect(screen.queryByRole('img', { name: '预览图' })).toBeNull()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    await vi.advanceTimersByTimeAsync(300)
    const current = pending.at(-1)
    expect(current?.id).toBe(second.id)
    await act(async () => {
      current?.resolve([
        first,
        task(second.id, second.displayName, {
          state: 'failed',
          preview: true,
        }),
      ])
      await Promise.resolve()
    })
    expect(screen.getByText('预览图生成失败')).toBeTruthy()
  })

  it('previews the current selection once when a config option changes', async () => {
    const existing = task('task-one', 'one.jpg')
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({ tasks: [existing] }),
    })
    const previewTask = vi
      .spyOn(fake.client, 'previewTask')
      .mockResolvedValue([existing])
    render(<App client={fake.client} />)
    await screen.findByRole('switch', { name: '实时预览' })
    vi.useFakeTimers()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
      await Promise.resolve()
    })
    await vi.advanceTimersByTimeAsync(300)
    expect(previewTask).toHaveBeenCalledTimes(1)
    previewTask.mockClear()

    await act(async () => {
      fireEvent.click(screen.getByRole('switch', { name: '纯色背景' }))
      await Promise.resolve()
    })
    await vi.advanceTimersByTimeAsync(300)

    expect(previewTask).toHaveBeenCalledTimes(1)
    expect(previewTask).toHaveBeenLastCalledWith(existing.id)
  })
})
