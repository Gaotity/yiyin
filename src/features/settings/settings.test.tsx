import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { App } from '../../app/App'
import {
  createFakePlatformClient,
  defaultBootstrap,
  type FakePlatformHarness,
} from '../../platform/fake'
import type {
  BootstrapDto,
  FontSpecDto,
  PublicConfigDto,
  TemplateDto,
  TemplateFieldDto,
} from '../../platform/types'
import { RenderingSettings } from './RenderingSettings'

afterEach(cleanup)

const DEFAULT_FONT: FontSpecDto = {
  family: '',
  size: 2.2,
  bold: false,
  italic: false,
  caseConversion: 'default',
  color: '',
}

const MODEL_FIELD: TemplateFieldDto = {
  key: 'Model',
  name: '型号',
  visible: true,
  useCustomValue: false,
  forceCustomValue: false,
  customValue: '',
  contentKind: 'text',
  darkImageId: null,
  lightImageId: null,
  fontOverride: null,
}

const MAKE_FIELD: TemplateFieldDto = {
  ...MODEL_FIELD,
  key: 'Make',
  name: 'Logo',
}

const SYSTEM_TEMPLATE: TemplateDto = {
  key: 'make-model',
  name: 'Logo型号模版',
  format: '{Make} {Model}',
  enabled: true,
  kind: 'system',
  height: null,
  font: { ...DEFAULT_FONT, size: 3, bold: true },
  verticalAlign: 'baseline',
}

const CUSTOM_TEMPLATE: TemplateDto = {
  key: 'custom-existing',
  name: '现有自定义模板',
  format: '{Model}',
  enabled: true,
  kind: 'custom',
  height: null,
  font: DEFAULT_FONT,
  verticalAlign: 'baseline',
}

const HELP_TEXT = [
  [
    '主图占比',
    '指定主图对背景宽度的占比（可以调节左右边框的宽度）\n默认主图占背景的90%',
  ],
  ['文本间距', '指定文本上下间距（临时性功能，后续会去掉）\n默认0.4'],
  [
    '最小上下边距',
    '指定水印上下边距的最小值，默认情况使用阴影宽度作为上下边距\n设置最小上下边距，将会从它和阴影之间取最大值\n按照背景高度比例换算，值为 0-100\n默认：0',
  ],
  ['圆角大小', '指定圆角的大小，不指定则为直角\n取值范围: 0 - 50\n默认值: 2.1'],
  [
    '阴影大小',
    '指定阴影的大小，不指定则无阴影\n设置的值为图片高度的百分比，例如: 1，则为0.01%\n默认值：6',
  ],
  ['输出质量', '指定输出质量，只允许整数\n默认值：100'],
  ['背景模糊', '指定背景图片的模糊程度\n取值范围: 0 - 100\n默认值: 100'],
  [
    '输出宽高比',
    '指定输出的图片的宽高比(该比例只生效于背景，对原图不生效)\n该选项生效后影响以下选项效果：\n横屏输出：失效',
  ],
  ['纯色背景', '使用纯色背景，默认使用图片模糊做背景'],
  [
    '横屏输出',
    '软件自己判断图片宽高那一边更长\n将背景横向处理\n适合竖图生成横屏图片',
  ],
  [
    '快速输出',
    '开启后选择图片/拖拽图片到软件将直接输出水印图片无需点击生成按钮',
  ],
  ['实时预览', '开启后点击列表图片可实时预览水印效果'],
] as const

function bootstrap(overrides: Partial<BootstrapDto> = {}): BootstrapDto {
  const base = defaultBootstrap()
  return {
    ...base,
    config: {
      ...base.config,
      templateFields: [MAKE_FIELD, MODEL_FIELD],
      customTemplateFields: [],
      templates: [SYSTEM_TEMPLATE, CUSTOM_TEMPLATE],
    },
    ...overrides,
  }
}

function renderApp(snapshot = bootstrap()): FakePlatformHarness {
  const fake = createFakePlatformClient({ snapshot })
  render(<App client={fake.client} />)
  return fake
}

function lastConfig(fake: FakePlatformHarness): PublicConfigDto {
  const request = fake.calls.updateConfig.at(-1)
  if (!request) {
    throw new Error('Expected an update_config call')
  }
  return request.config
}

async function changeNumber(label: string, value: string) {
  const input = screen.getByLabelText(label)
  fireEvent.change(input, { target: { value } })
  fireEvent.blur(input)
}

describe('settings parity', () => {
  it('preserves every established rendering help message', async () => {
    renderApp()
    await screen.findByText('主图占比')

    for (const [label, help] of HELP_TEXT) {
      const heading = screen
        .getAllByText(label)
        .find((element) => element.tagName === 'STRONG')
      const row = heading?.closest('.setting-row')
      expect(row?.querySelector('small')?.textContent).toBe(help)
    }
  })

  it('shows the established controls and canonicalizes numeric values before saving', async () => {
    const fake = renderApp()
    await screen.findByText('主图占比')

    expect(
      screen.getByText('主图占比').closest('.setting-row')?.textContent,
    ).toContain('指定主图对背景宽度的占比（可以调节左右边框的宽度）')
    expect(screen.getByLabelText('输出质量')).toHaveProperty('value', '100')

    await changeNumber('主图占比', '0')
    await waitFor(() => expect(lastConfig(fake).options.mainImageWidth).toBe(1))

    await changeNumber('圆角大小', '3.66')
    await waitFor(() => expect(lastConfig(fake).options.radius).toBe(3.7))

    await changeNumber('输出质量', '101')
    await waitFor(() => expect(lastConfig(fake).options.quality).toBe(100))

    await changeNumber('背景模糊', '3.6')
    await waitFor(() => expect(lastConfig(fake).options.backgroundBlur).toBe(4))
  })

  it('swaps the output ratio, disables landscape, and persists display toggles immediately', async () => {
    const snapshot = bootstrap()
    snapshot.config.options.landscape = true
    snapshot.config.options.backgroundRatio = { width: 3, height: 2 }
    const fake = renderApp(snapshot)
    await screen.findByRole('switch', { name: '输出宽高比' })

    fireEvent.click(screen.getByRole('switch', { name: '输出宽高比' }))
    await waitFor(() => {
      expect(lastConfig(fake).options.backgroundRatioVisible).toBe(true)
      expect(lastConfig(fake).options.landscape).toBe(false)
    })
    expect(
      screen
        .getByRole('switch', { name: '横屏输出' })
        .getAttribute('aria-disabled'),
    ).toBe('true')

    fireEvent.click(screen.getByRole('button', { name: '交换宽高比' }))
    await waitFor(() => {
      expect(lastConfig(fake).options.backgroundRatio).toEqual({
        width: 2,
        height: 3,
      })
    })

    fireEvent.click(screen.getByRole('switch', { name: '纯色背景' }))
    fireEvent.change(await screen.findByLabelText('背景颜色'), {
      target: { value: '#123456' },
    })
    fireEvent.click(screen.getByRole('switch', { name: '快速输出' }))
    fireEvent.click(screen.getByRole('switch', { name: '实时预览' }))
    await waitFor(() => {
      const config = lastConfig(fake)
      expect(config.options.solidColor).toBe('#123456')
      expect(config.options.quickOutput).toBe(true)
      expect(config.options.previewVisible).toBe(true)
    })
  })

  it('treats field edits as transactions and preserves custom keys', async () => {
    const snapshot = bootstrap()
    snapshot.config.customTemplateFields = [
      { ...MODEL_FIELD, key: 'custom-stable', name: '自定义字段' },
    ]
    const fake = renderApp(snapshot)
    await screen.findByText('参数设置')
    fireEvent.click(screen.getByRole('button', { name: '参数设置' }))

    expect(await screen.findByText('相机参数')).toBeTruthy()
    expect(screen.queryByRole('button', { name: '删除Logo' })).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: '编辑型号' }))
    fireEvent.change(await screen.findByLabelText('文本内容'), {
      target: { value: 'cancelled' },
    })
    fireEvent.click(screen.getByRole('button', { name: '取消' }))
    expect(fake.calls.updateConfig).toHaveLength(0)

    fireEvent.click(screen.getByRole('button', { name: '编辑自定义字段' }))
    fireEvent.click(await screen.findByRole('switch', { name: '是否生效' }))
    fireEvent.click(screen.getByRole('switch', { name: '强制替换' }))
    fireEvent.change(screen.getByLabelText('文本内容'), {
      target: { value: 'stable value' },
    })
    fireEvent.click(screen.getByRole('button', { name: '保存' }))
    await waitFor(() => {
      const field = lastConfig(fake).customTemplateFields[0]
      expect(field?.key).toBe('custom-stable')
      expect(field?.customValue).toBe('stable value')
      expect(field?.forceCustomValue).toBe(true)
    })
  })

  it('registers opaque light and dark overlays for image fields', async () => {
    const fake = renderApp()
    const registerOverlay = vi.spyOn(fake.client, 'registerOverlay')
    await screen.findByText('参数设置')
    fireEvent.click(screen.getByRole('button', { name: '参数设置' }))
    fireEvent.click(await screen.findByRole('button', { name: '编辑Logo' }))
    fireEvent.click(await screen.findByRole('radio', { name: '图片' }))
    fireEvent.click(screen.getByRole('button', { name: '选择模糊背景图片' }))
    fireEvent.click(screen.getByRole('button', { name: '选择纯色背景图片' }))
    await waitFor(() => expect(registerOverlay).toHaveBeenCalledTimes(2))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    await waitFor(() => {
      const field = lastConfig(fake).templateFields.find(
        (item) => item.key === 'Make',
      )
      expect(field?.contentKind).toBe('image')
      expect(field?.lightImageId).toMatch(/^overlay-/)
      expect(field?.darkImageId).toMatch(/^overlay-/)
      expect(JSON.stringify(field)).not.toContain('/Users/')
    })
  })

  it('protects system templates and commits custom placeholder and alignment changes only on save', async () => {
    const fake = renderApp()
    await screen.findByText('模板设置')
    fireEvent.click(screen.getByRole('button', { name: '模板设置' }))
    expect((await screen.findAllByText('文本模板设置')).length).toBeGreaterThan(
      0,
    )
    expect(
      screen.queryByRole('button', { name: '删除Logo型号模版' }),
    ).toBeNull()

    fireEvent.click(screen.getByRole('button', { name: '编辑Logo型号模版' }))
    fireEvent.change(await screen.findByLabelText('名称'), {
      target: { value: 'cancelled' },
    })
    fireEvent.click(screen.getByRole('button', { name: '取消' }))
    expect(fake.calls.updateConfig).toHaveLength(0)

    fireEvent.click(screen.getByRole('button', { name: '添加文字模板' }))
    fireEvent.change(await screen.findByLabelText('名称'), {
      target: { value: '自定义模板' },
    })
    fireEvent.click(screen.getByRole('button', { name: '插入 型号' }))
    fireEvent.click(screen.getByRole('radio', { name: '居中对齐' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))

    await waitFor(() => {
      const template = lastConfig(fake).templates.at(-1)
      expect(template?.key).toMatch(/^custom-template-/)
      expect(template?.format).toContain('{Model}')
      expect(template?.verticalAlign).toBe('center')
      expect(template?.kind).toBe('custom')
    })
  })

  it('keeps bundled fonts, registers custom fonts, and falls back after deleting the selected font', async () => {
    const snapshot = bootstrap({
      resources: [
        {
          id: 'font-custom',
          kind: 'font',
          displayName: 'Custom Sans',
          url: 'yiyin://resource/font-custom',
        },
      ],
    })
    snapshot.config.options.font = 'Custom Sans'
    const fake = renderApp(snapshot)
    const registerFont = vi.spyOn(fake.client, 'registerFont')
    const removeFont = vi.spyOn(fake.client, 'removeFont')
    await screen.findByRole('combobox', { name: '字体' })

    fireEvent.click(screen.getByRole('combobox', { name: '字体' }))
    expect(await screen.findByRole('option', { name: '春风楷' })).toBeTruthy()
    expect(screen.getByRole('option', { name: '千图小兔' })).toBeTruthy()
    expect(
      screen
        .getByRole('button', { name: '删除字体 Custom Sans' })
        .closest('[role="option"]'),
    ).not.toBeNull()
    expect(
      screen
        .getByRole('button', { name: '添加字体' })
        .closest('.select-surface'),
    ).not.toBeNull()

    fireEvent.click(screen.getByRole('button', { name: '添加字体' }))
    fireEvent.change(await screen.findByLabelText('字体名称'), {
      target: { value: 'Fixture Sans' },
    })
    fireEvent.click(screen.getByRole('button', { name: '添加' }))
    await waitFor(() =>
      expect(registerFont).toHaveBeenCalledWith('Fixture Sans'),
    )
    await waitFor(() => expect(screen.queryByLabelText('字体名称')).toBeNull())

    let deleteFont = screen.queryByRole('button', {
      name: '删除字体 Custom Sans',
    })
    if (!deleteFont) {
      fireEvent.click(screen.getByRole('combobox', { name: '字体' }))
      deleteFont = await screen.findByRole('button', {
        name: '删除字体 Custom Sans',
      })
    }
    fireEvent.click(deleteFont)
    await waitFor(() => {
      expect(removeFont).toHaveBeenCalledWith('font-custom')
      expect(lastConfig(fake).options.font).toBe('PingFang SC')
    })
    expect(document.body.textContent).not.toContain('/Users/')
  })

  it('adds, edits, hides, and deletes a custom field without changing its generated key', async () => {
    const fake = renderApp()
    await screen.findByText('参数设置')
    fireEvent.click(screen.getByRole('button', { name: '参数设置' }))
    fireEvent.click(await screen.findByRole('switch', { name: '显示型号' }))
    await waitFor(() => {
      const model = lastConfig(fake).templateFields.find(
        (field) => field.key === 'Model',
      )
      expect(model?.visible).toBe(false)
    })

    fireEvent.click(screen.getByRole('button', { name: '添加自定义参数' }))
    fireEvent.change(await screen.findByLabelText('字段名称'), {
      target: { value: '作者' },
    })
    fireEvent.change(screen.getByLabelText('文本内容'), {
      target: { value: 'Yiyin' },
    })
    fireEvent.click(screen.getByRole('button', { name: '保存' }))
    await waitFor(() => {
      expect(lastConfig(fake).customTemplateFields[0]?.key).toMatch(
        /^custom-field-/,
      )
    })
    const key = lastConfig(fake).customTemplateFields[0]?.key

    fireEvent.click(screen.getByRole('button', { name: '编辑作者' }))
    fireEvent.change(await screen.findByLabelText('文本内容'), {
      target: { value: 'Yiyin 2' },
    })
    fireEvent.click(screen.getByRole('switch', { name: '自定义字体参数' }))
    fireEvent.change(screen.getByLabelText('字体大小'), {
      target: { value: '3.2' },
    })
    fireEvent.click(screen.getByRole('radio', { name: '大写' }))
    fireEvent.click(screen.getByRole('button', { name: '保存' }))
    await waitFor(() => {
      const field = lastConfig(fake).customTemplateFields[0]
      expect(field?.key).toBe(key)
      expect(field?.fontOverride?.size).toBe(3.2)
      expect(field?.fontOverride?.caseConversion).toBe('uppercase')
    })

    fireEvent.click(screen.getByRole('button', { name: '删除作者' }))
    await waitFor(() => {
      expect(lastConfig(fake).customTemplateFields).toHaveLength(0)
    })
  })

  it('reorders and deletes only custom templates', async () => {
    const fake = renderApp()
    await screen.findByText('模板设置')
    fireEvent.click(screen.getByRole('button', { name: '模板设置' }))
    fireEvent.click(
      await screen.findByRole('button', { name: '上移现有自定义模板' }),
    )
    await waitFor(() => {
      expect(lastConfig(fake).templates[0]?.key).toBe('custom-existing')
    })
    expect(
      screen.queryByRole('button', { name: '删除Logo型号模版' }),
    ).toBeNull()
    fireEvent.click(screen.getByRole('button', { name: '删除现有自定义模板' }))
    await waitFor(() => {
      expect(
        lastConfig(fake).templates.some(
          (template) => template.key === 'custom-existing',
        ),
      ).toBe(false)
    })
  })

  it('shows stable font errors without exposing native paths', async () => {
    const fake = createFakePlatformClient({ snapshot: bootstrap() })
    vi.spyOn(fake.client, 'registerFont')
      .mockRejectedValueOnce({
        code: 'INVALID_REQUEST',
        message: 'A font with this name already exists.',
      })
      .mockRejectedValueOnce({
        code: 'FILE_NOT_FOUND',
        message: 'The selected file was not found.',
      })
    render(<App client={fake.client} />)
    await screen.findByRole('combobox', { name: '字体' })
    fireEvent.click(screen.getByRole('combobox', { name: '字体' }))
    fireEvent.click(screen.getByRole('button', { name: '添加字体' }))
    fireEvent.change(await screen.findByLabelText('字体名称'), {
      target: { value: 'Duplicate' },
    })
    fireEvent.click(screen.getByRole('button', { name: '添加' }))
    expect(
      await screen.findByText('A font with this name already exists.'),
    ).toBeTruthy()
    fireEvent.click(screen.getByRole('button', { name: '添加' }))
    expect(
      await screen.findByText('The selected file was not found.'),
    ).toBeTruthy()
    expect(document.body.textContent).not.toContain('/Users/')
  })

  it('debounces continuous background blur saves and flushes the latest value', async () => {
    vi.useFakeTimers()
    const config = bootstrap().config
    const save = vi.fn(async (next: PublicConfigDto) => next)
    const view = render(
      <RenderingSettings
        config={config}
        constraints={bootstrap().constraints}
        onSave={save}
      />,
    )
    const slider = screen.getByLabelText('背景模糊滑块')

    fireEvent.change(slider, { target: { value: '20' } })
    fireEvent.change(slider, { target: { value: '21' } })
    await vi.advanceTimersByTimeAsync(299)
    expect(save).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(1)
    expect(save).toHaveBeenCalledTimes(1)
    expect(save.mock.calls[0]?.[0].options.backgroundBlur).toBe(21)

    view.unmount()
    vi.useRealTimers()
  })

  it('clamps with the bootstrap constraints prop, never a local table', async () => {
    const config = bootstrap().config
    const constraints = {
      ...bootstrap().constraints,
      // Deliberately not the domain truth: clamping must follow the prop.
      quality: { minimum: 5, maximum: 42, decimals: 0 },
      backgroundBlur: { minimum: 10, maximum: 55, decimals: 0 },
    }
    const save = vi.fn(async (next: PublicConfigDto) => next)
    render(
      <RenderingSettings
        config={config}
        constraints={constraints}
        onSave={save}
      />,
    )

    fireEvent.change(screen.getByLabelText('输出质量'), {
      target: { value: '100' },
    })
    fireEvent.blur(screen.getByLabelText('输出质量'))
    await waitFor(() => expect(save).toHaveBeenCalled())
    expect(save.mock.calls[0]?.[0].options.quality).toBe(42)

    fireEvent.change(screen.getByLabelText('输出质量'), {
      target: { value: '1' },
    })
    fireEvent.blur(screen.getByLabelText('输出质量'))
    await waitFor(() => expect(save).toHaveBeenCalledTimes(2))
    expect(save.mock.calls[1]?.[0].options.quality).toBe(5)

    const slider = screen.getByLabelText('背景模糊滑块')
    expect(slider.getAttribute('min')).toBe('10')
    expect(slider.getAttribute('max')).toBe('55')
    expect(slider.getAttribute('step')).toBe('1')
  })
})
