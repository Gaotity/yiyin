import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { createFakePlatformClient, defaultBootstrap } from '../platform/fake'
import { App } from './App'

afterEach(cleanup)

describe('App', () => {
  it('shows loading, then the authoritative bootstrap snapshot', async () => {
    const snapshot = defaultBootstrap({
      tasks: [
        {
          id: 'task-1',
          displayName: 'photo.jpg',
          state: 'registered',
          progress: 0,
          preview: false,
          resource: null,
        },
      ],
    })
    const fake = createFakePlatformClient({ snapshot, deferredBootstrap: true })
    render(<App client={fake.client} />)

    expect(screen.getByText('正在加载…')).toBeTruthy()
    await act(async () => fake.resolveBootstrap())

    expect(await screen.findByText('photo.jpg')).toBeTruthy()
    expect(screen.getByText('等待处理')).toBeTruthy()
  })

  it('presents migration warnings and a safe fatal bootstrap error', async () => {
    const warning = createFakePlatformClient({
      snapshot: defaultBootstrap({
        warnings: ['Skipped one invalid legacy resource.'],
      }),
    })
    const warningView = render(<App client={warning.client} />)
    expect(await screen.findByText('迁移提示')).toBeTruthy()
    expect(
      screen.getByText('Skipped one invalid legacy resource.'),
    ).toBeTruthy()
    warningView.unmount()

    const failed = createFakePlatformClient()
    failed.rejectNextBootstrap({
      code: 'CONFIG_INVALID',
      message: 'The configuration is invalid.',
    })
    render(<App client={failed.client} />)
    expect(await screen.findByText('壹印启动失败')).toBeTruthy()
    expect(screen.getByText('The configuration is invalid.')).toBeTruthy()
  })

  it('reconciles a task event received before bootstrap resolves', async () => {
    const snapshot = defaultBootstrap({
      tasks: [
        {
          id: 'task-1',
          displayName: 'photo.jpg',
          state: 'registered',
          progress: 0,
          preview: false,
          resource: null,
        },
      ],
    })
    const fake = createFakePlatformClient({ snapshot, deferredBootstrap: true })
    render(<App client={fake.client} />)

    act(() => {
      fake.emitTaskStatus({
        taskId: 'task-1',
        state: 'running',
        progress: 30,
        preview: false,
        cancellationReason: null,
      })
    })
    await act(async () => fake.resolveBootstrap())

    expect(await screen.findByText('处理中')).toBeTruthy()
    expect(screen.getByText('30%')).toBeTruthy()
  })

  it('re-bootstraps for an unknown task event and unsubscribes on unmount', async () => {
    const fake = createFakePlatformClient()
    const view = render(<App client={fake.client} />)
    await screen.findByText('添加图片')

    act(() => {
      fake.emitTaskStatus({
        taskId: 'unknown-task',
        state: 'running',
        progress: 10,
        preview: false,
        cancellationReason: null,
      })
    })
    await waitFor(() => expect(fake.calls.bootstrap).toBe(2))

    view.unmount()
    expect(fake.calls.unlisten).toBe(1)
  })

  it('preserves help, feedback, donation, external link, and window interactions', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    Object.defineProperty(navigator, 'clipboard', {
      configurable: true,
      value: { writeText },
    })
    const fake = createFakePlatformClient({
      snapshot: defaultBootstrap({
        resources: [
          {
            id: 'donate-wechat',
            kind: 'bundledAsset',
            displayName: 'zs-wx.jpg',
            url: 'yiyin://resource/donate-wechat',
          },
          {
            id: 'donate-alipay',
            kind: 'bundledAsset',
            displayName: 'zs-zfb.jpg',
            url: 'yiyin://resource/donate-alipay',
          },
        ],
      }),
    })
    render(<App client={fake.client} />)
    await screen.findByText('添加图片')

    fireEvent.click(screen.getByRole('button', { name: '帮助' }))
    expect(await screen.findByText('一些常见问题解答')).toBeTruthy()
    expect(screen.getByText('为什么输出的水印没有相机参数？')).toBeTruthy()
    expect(
      screen.getByText(
        '请注意，如果您在Photoshop中对图片进行了大量编辑，可能会导致某些EXIF信息（如镜头信息、曝光时间等）不再准确反映编辑后的图片状态。此外，如果您使用的是Photoshop的“保存为Web所用格式”(Save for Web)功能或较早版本的Photoshop，保留EXIF信息的步骤可能会有所不同。',
      ),
    ).toBeTruthy()

    fireEvent.click(screen.getByRole('button', { name: '反馈与赞赏' }))
    fireEvent.click(
      await screen.findByRole('button', { name: 'QQ交流群:718615618' }),
    )
    expect(writeText).toHaveBeenCalledWith('718615618')
    expect(screen.getByAltText('微信赞赏码').getAttribute('src')).toBe(
      'yiyin://resource/donate-wechat',
    )
    expect(screen.getByAltText('支付宝赞赏码').getAttribute('src')).toBe(
      'yiyin://resource/donate-alipay',
    )

    fireEvent.click(
      screen.getByRole('button', { name: '反馈 - 建议(Github Issues)' }),
    )
    fireEvent.click(screen.getByRole('button', { name: 'v1.6.0' }))
    fireEvent.click(
      screen.getByRole('button', { name: 'B站 - 不长肉的小伙吒' }),
    )
    fireEvent.click(
      screen.getByRole('button', { name: '© 2023 Github - GGChivalrous.' }),
    )
    expect(fake.calls.openExternalUrl).toEqual([
      'issues',
      'currentRelease',
      'bilibiliProfile',
      'repository',
    ])

    fireEvent.click(screen.getByRole('button', { name: '重置回默认选项' }))
    fireEvent.click(screen.getByRole('button', { name: '最小化' }))
    fireEvent.click(screen.getByRole('button', { name: '关闭' }))
    expect(fake.calls.resetConfig).toBe(1)
    expect(fake.calls.minimizeWindow).toBe(1)
    expect(fake.calls.closeWindow).toBe(1)
  })
})
