import { useEffect, useRef, useState, type ReactNode } from 'react'
import { Switch } from '../../components/Switch'
import type {
  BootstrapDto,
  NumericConstraintDto,
  PublicConfigDto,
} from '../../platform/types'
import {
  setBackgroundRatioHeight,
  setBackgroundRatioVisible,
  setBackgroundRatioWidth,
  setLandscape,
  setNumberOption,
  setPreviewVisible,
  setQuickOutput,
  setRadiusVisible,
  setShadowVisible,
  setSolidBackground,
  setSolidColor,
  swapBackgroundRatio,
} from './configIntents'

interface RenderingSettingsProps {
  config: PublicConfigDto
  constraints: BootstrapDto['constraints']
  onSave(config: PublicConfigDto): Promise<PublicConfigDto>
}

type NumericOptionKey = keyof BootstrapDto['constraints']

const UNBOUNDED: NumericConstraintDto = {
  minimum: -Number.MAX_SAFE_INTEGER,
  maximum: Number.MAX_SAFE_INTEGER,
  decimals: 3,
}

interface NumberSetting {
  key: NumericOptionKey
  label: string
  help: string
}

const NUMBER_SETTINGS: NumberSetting[] = [
  {
    key: 'mainImageWidth',
    label: '主图占比',
    help: '指定主图对背景宽度的占比（可以调节左右边框的宽度）\n默认主图占背景的90%',
  },
  {
    key: 'textMargin',
    label: '文本间距',
    help: '指定文本上下间距（临时性功能，后续会去掉）\n默认0.4',
  },
  {
    key: 'miniTopBottomMargin',
    label: '最小上下边距',
    help: '指定水印上下边距的最小值，默认情况使用阴影宽度作为上下边距\n设置最小上下边距，将会从它和阴影之间取最大值\n按照背景高度比例换算，值为 0-100\n默认：0',
  },
  {
    key: 'radius',
    label: '圆角大小',
    help: '指定圆角的大小，不指定则为直角\n取值范围: 0 - 50\n默认值: 2.1',
  },
  {
    key: 'shadow',
    label: '阴影大小',
    help: '指定阴影的大小，不指定则无阴影\n设置的值为图片高度的百分比，例如: 1，则为0.01%\n默认值：6',
  },
  {
    key: 'quality',
    label: '输出质量',
    help: '指定输出质量，只允许整数\n默认值：100',
  },
  {
    key: 'backgroundBlur',
    label: '背景模糊',
    help: '指定背景图片的模糊程度\n取值范围: 0 - 100\n默认值: 100',
  },
]

export function RenderingSettings({
  config,
  constraints,
  onSave,
}: RenderingSettingsProps) {
  const [draft, setDraft] = useState(config)
  const [error, setError] = useState('')
  const blurTimer = useRef<number | null>(null)
  const pendingBlur = useRef<PublicConfigDto | null>(null)
  const onSaveRef = useRef(onSave)
  const blurConstraint = constraints.backgroundBlur ?? UNBOUNDED

  useEffect(() => setDraft(config), [config])
  useEffect(() => {
    onSaveRef.current = onSave
  }, [onSave])
  useEffect(
    () => () => {
      if (blurTimer.current !== null) {
        window.clearTimeout(blurTimer.current)
      }
      if (pendingBlur.current) {
        void onSaveRef.current(pendingBlur.current)
      }
    },
    [],
  )

  const commit = async (next: PublicConfigDto) => {
    setDraft(next)
    setError('')
    try {
      setDraft(await onSave(next))
    } catch (reason) {
      setError(readMessage(reason))
      setDraft(config)
    }
  }

  const saveNumber = (setting: NumberSetting, raw: string) => {
    const constraint = constraints[setting.key] ?? UNBOUNDED
    const value = canonicalNumber(
      raw,
      constraint.minimum,
      constraint.maximum,
      constraint.decimals,
    )
    void commit(setNumberOption(draft, setting.key, value))
  }

  const scheduleBlur = (raw: string) => {
    const value = canonicalNumber(
      raw,
      blurConstraint.minimum,
      blurConstraint.maximum,
      blurConstraint.decimals,
    )
    const next = setNumberOption(draft, 'backgroundBlur', value)
    setDraft(next)
    pendingBlur.current = next
    if (blurTimer.current !== null) {
      window.clearTimeout(blurTimer.current)
    }
    blurTimer.current = window.setTimeout(() => {
      const pending = pendingBlur.current
      pendingBlur.current = null
      blurTimer.current = null
      if (pending) {
        void commit(pending)
      }
    }, 300)
  }

  const flushBlur = () => {
    if (blurTimer.current !== null) {
      window.clearTimeout(blurTimer.current)
      blurTimer.current = null
    }
    const pending = pendingBlur.current
    pendingBlur.current = null
    if (pending) {
      void commit(pending)
    }
  }

  return (
    <section className="rendering-settings" aria-label="渲染设置">
      {NUMBER_SETTINGS.map((setting) => (
        <SettingRow key={setting.key} label={setting.label} help={setting.help}>
          {setting.key === 'radius' && (
            <Switch
              checked={draft.options.radiusVisible}
              onCheckedChange={(checked) =>
                void commit(setRadiusVisible(draft, checked))
              }
              label="启用圆角"
            />
          )}
          {setting.key === 'shadow' && (
            <Switch
              checked={draft.options.shadowVisible}
              onCheckedChange={(checked) =>
                void commit(setShadowVisible(draft, checked))
              }
              label="启用阴影"
            />
          )}
          {setting.key === 'backgroundBlur' && (
            <input
              aria-label="背景模糊滑块"
              type="range"
              min={blurConstraint.minimum}
              max={blurConstraint.maximum}
              step={10 ** -blurConstraint.decimals}
              value={draft.options.backgroundBlur}
              onChange={(event) => scheduleBlur(event.currentTarget.value)}
              onBlur={flushBlur}
            />
          )}
          <NumericInput
            label={setting.label}
            className="setting-number"
            value={draft.options[setting.key]}
            onCommit={(value) => saveNumber(setting, value)}
          />
        </SettingRow>
      ))}

      <SettingRow
        label="输出宽高比"
        help={
          '指定输出的图片的宽高比(该比例只生效于背景，对原图不生效)\n该选项生效后影响以下选项效果：\n横屏输出：失效'
        }
      >
        <Switch
          checked={draft.options.backgroundRatioVisible}
          onCheckedChange={(checked) =>
            void commit(setBackgroundRatioVisible(draft, checked))
          }
          label="输出宽高比"
        />
        <NumericInput
          label="宽高比宽度"
          className="ratio-number"
          value={draft.options.backgroundRatio.width}
          onCommit={(value) =>
            void commit(
              setBackgroundRatioWidth(
                draft,
                canonicalNumber(value, 0, Number.MAX_SAFE_INTEGER, 3),
              ),
            )
          }
        />
        <button
          type="button"
          aria-label="交换宽高比"
          onClick={() => void commit(swapBackgroundRatio(draft))}
        >
          ⇄
        </button>
        <NumericInput
          label="宽高比高度"
          className="ratio-number"
          value={draft.options.backgroundRatio.height}
          onCommit={(value) =>
            void commit(
              setBackgroundRatioHeight(
                draft,
                canonicalNumber(value, 0, Number.MAX_SAFE_INTEGER, 3),
              ),
            )
          }
        />
      </SettingRow>

      <BooleanSetting
        label="纯色背景"
        help="使用纯色背景，默认使用图片模糊做背景"
        checked={draft.options.solidBackground}
        onChange={(checked) => void commit(setSolidBackground(draft, checked))}
      >
        {draft.options.solidBackground && (
          <input
            aria-label="背景颜色"
            type="color"
            value={expandColor(draft.options.solidColor)}
            onChange={(event) =>
              void commit(setSolidColor(draft, event.currentTarget.value))
            }
          />
        )}
      </BooleanSetting>
      <BooleanSetting
        label="横屏输出"
        help={
          '软件自己判断图片宽高那一边更长\n将背景横向处理\n适合竖图生成横屏图片'
        }
        checked={draft.options.landscape}
        disabled={draft.options.backgroundRatioVisible}
        onChange={(checked) => void commit(setLandscape(draft, checked))}
      />
      <BooleanSetting
        label="快速输出"
        help="开启后选择图片/拖拽图片到软件将直接输出水印图片无需点击生成按钮"
        checked={draft.options.quickOutput}
        onChange={(checked) => void commit(setQuickOutput(draft, checked))}
      />
      <BooleanSetting
        label="实时预览"
        help="开启后点击列表图片可实时预览水印效果"
        checked={draft.options.previewVisible}
        onChange={(checked) => void commit(setPreviewVisible(draft, checked))}
      />
      {error && <p role="alert">{error}</p>}
    </section>
  )
}

interface SettingRowProps {
  label: string
  help: string
  children: ReactNode
}

function SettingRow({ label, help, children }: SettingRowProps) {
  return (
    <div className="setting-row">
      <div className="setting-copy">
        <strong>{label}</strong>
        <small>{help}</small>
      </div>
      <div className="setting-control">{children}</div>
    </div>
  )
}

interface BooleanSettingProps {
  label: string
  help: string
  checked: boolean
  disabled?: boolean
  onChange(checked: boolean): void
  children?: ReactNode
}

function BooleanSetting({
  label,
  help,
  checked,
  disabled = false,
  onChange,
  children,
}: BooleanSettingProps) {
  return (
    <SettingRow label={label} help={help}>
      <Switch
        checked={checked}
        disabled={disabled}
        onCheckedChange={onChange}
        label={label}
      />
      {children}
    </SettingRow>
  )
}

interface NumericInputProps {
  label: string
  className: string
  value: number
  onCommit(value: string): void
}

function NumericInput({
  label,
  className,
  value,
  onCommit,
}: NumericInputProps) {
  const [input, setInput] = useState(String(value))

  useEffect(() => setInput(String(value)), [value])

  return (
    <input
      aria-label={label}
      className={className}
      inputMode="decimal"
      value={input}
      onChange={(event) => setInput(event.currentTarget.value)}
      onBlur={() => onCommit(input)}
    />
  )
}

function canonicalNumber(
  raw: string,
  minimum: number,
  maximum: number,
  decimals: number,
): number {
  const parsed = Number.parseFloat(raw.match(/-?\d+(?:\.\d{0,3})?/)?.[0] ?? '')
  const bounded = Number.isFinite(parsed)
    ? Math.min(maximum, Math.max(minimum, parsed))
    : minimum
  const scale = 10 ** decimals
  return Math.round(bounded * scale) / scale
}

function expandColor(value: string): string {
  if (/^#[0-9a-f]{6}$/i.test(value)) {
    return value
  }
  if (/^#[0-9a-f]{3}$/i.test(value)) {
    return `#${value
      .slice(1)
      .split('')
      .map((digit) => `${digit}${digit}`)
      .join('')}`
  }
  return '#ffffff'
}

function readMessage(reason: unknown): string {
  return typeof reason === 'object' && reason !== null && 'message' in reason
    ? String(reason.message)
    : '配置保存失败'
}
