import { describe, expect, it } from 'vitest'
import { defaultBootstrap } from '../../platform/fake'
import type {
  FontSpecDto,
  PublicConfigDto,
  TemplateDto,
  TemplateFieldDto,
} from '../../platform/types'
import {
  moveTemplate,
  removeCustomTemplateField,
  removeTemplate,
  setBackgroundRatioHeight,
  setBackgroundRatioVisible,
  setBackgroundRatioWidth,
  setFont,
  setLandscape,
  setNumberOption,
  setPreviewVisible,
  setQuickOutput,
  setRadiusVisible,
  setShadowVisible,
  setSolidBackground,
  setSolidColor,
  swapBackgroundRatio,
  upsertTemplate,
  upsertTemplateField,
} from './configIntents'

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

const CUSTOM_FIELD: TemplateFieldDto = {
  ...MODEL_FIELD,
  key: 'custom-note',
  name: '备注',
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

function baseConfig(): PublicConfigDto {
  return {
    ...defaultBootstrap().config,
    templateFields: [structuredClone(MODEL_FIELD)],
    customTemplateFields: [structuredClone(CUSTOM_FIELD)],
    templates: [
      structuredClone(SYSTEM_TEMPLATE),
      structuredClone(CUSTOM_TEMPLATE),
    ],
  }
}

describe('configIntents', () => {
  it('setNumberOption replaces only the addressed option without mutating the input', () => {
    const config = baseConfig()

    const next = setNumberOption(config, 'quality', 42)

    expect(next.options.quality).toBe(42)
    expect(next.options.radius).toBe(config.options.radius)
    expect(config.options.quality).toBe(100)
  })

  it('setBackgroundRatioVisible carries the landscape coupling only when enabling', () => {
    const config = baseConfig()
    config.options.landscape = true

    const enabled = setBackgroundRatioVisible(config, true)

    expect(enabled.options.backgroundRatioVisible).toBe(true)
    expect(enabled.options.landscape).toBe(false)
    expect(config.options.backgroundRatioVisible).toBe(false)
    expect(config.options.landscape).toBe(true)

    // A contradictory state persisted before the domain rule existed:
    // disabling the guide must not touch landscape.
    const stale = baseConfig()
    stale.options.backgroundRatioVisible = true
    stale.options.landscape = true

    const disabled = setBackgroundRatioVisible(stale, false)

    expect(disabled.options.backgroundRatioVisible).toBe(false)
    expect(disabled.options.landscape).toBe(true)
  })

  it('applies the plain switch and value edits verbatim', () => {
    const config = baseConfig()

    expect(setLandscape(config, true).options.landscape).toBe(true)
    expect(setRadiusVisible(config, false).options.radiusVisible).toBe(false)
    expect(setShadowVisible(config, false).options.shadowVisible).toBe(false)
    expect(setSolidBackground(config, true).options.solidBackground).toBe(true)
    expect(setSolidColor(config, '#123456').options.solidColor).toBe('#123456')
    expect(setQuickOutput(config, true).options.quickOutput).toBe(true)
    expect(setPreviewVisible(config, true).options.previewVisible).toBe(true)
    expect(setFont(config, 'Fixture Sans').options.font).toBe('Fixture Sans')
    expect(config.options.landscape).toBe(false)
    expect(config.options.font).toBe('PingFang SC')
  })

  it('edits the background ratio dimensions and swaps them', () => {
    const config = baseConfig()
    config.options.backgroundRatio = { width: 3, height: 2 }

    expect(setBackgroundRatioWidth(config, 4).options.backgroundRatio).toEqual({
      width: 4,
      height: 2,
    })
    expect(setBackgroundRatioHeight(config, 5).options.backgroundRatio).toEqual(
      { width: 3, height: 5 },
    )
    expect(swapBackgroundRatio(config).options.backgroundRatio).toEqual({
      width: 2,
      height: 3,
    })
    expect(config.options.backgroundRatio).toEqual({ width: 3, height: 2 })
  })

  it('upsertTemplateField replaces by key or appends in the addressed list', () => {
    const config = baseConfig()

    const replaced = upsertTemplateField(
      config,
      { ...MODEL_FIELD, visible: false },
      false,
    )
    expect(replaced.templateFields[0]?.visible).toBe(false)
    expect(replaced.customTemplateFields).toEqual(config.customTemplateFields)

    const added = upsertTemplateField(
      config,
      { ...CUSTOM_FIELD, key: 'custom-extra' },
      true,
    )
    expect(added.customTemplateFields.map((field) => field.key)).toEqual([
      'custom-note',
      'custom-extra',
    ])
    expect(added.templateFields).toEqual(config.templateFields)

    expect(config.templateFields[0]?.visible).toBe(true)
    expect(config.customTemplateFields).toHaveLength(1)
  })

  it('removeCustomTemplateField drops only the addressed custom field', () => {
    const config = baseConfig()

    const next = removeCustomTemplateField(config, 'custom-note')

    expect(next.customTemplateFields).toHaveLength(0)
    expect(next.templateFields).toEqual(config.templateFields)
    expect(config.customTemplateFields).toHaveLength(1)
    expect(removeCustomTemplateField(config, 'missing')).toBe(config)
  })

  it('upsertTemplate replaces by key or appends', () => {
    const config = baseConfig()

    const replaced = upsertTemplate(config, {
      ...CUSTOM_TEMPLATE,
      enabled: false,
    })
    expect(
      replaced.templates.find((template) => template.key === 'custom-existing')
        ?.enabled,
    ).toBe(false)
    expect(replaced.templates).toHaveLength(2)

    const added = upsertTemplate(config, {
      ...CUSTOM_TEMPLATE,
      key: 'custom-new',
    })
    expect(added.templates.map((template) => template.key)).toEqual([
      'make-model',
      'custom-existing',
      'custom-new',
    ])
    expect(config.templates).toHaveLength(2)
  })

  it('removeTemplate refuses system templates and removes custom ones', () => {
    const config = baseConfig()

    expect(removeTemplate(config, SYSTEM_TEMPLATE)).toBe(config)
    expect(removeTemplate(config, { ...CUSTOM_TEMPLATE, key: 'missing' })).toBe(
      config,
    )

    const next = removeTemplate(config, CUSTOM_TEMPLATE)
    expect(next.templates.map((template) => template.key)).toEqual([
      'make-model',
    ])
    expect(config.templates).toHaveLength(2)
  })

  it('moveTemplate swaps adjacent templates only within bounds', () => {
    const config = baseConfig()

    const moved = moveTemplate(config, 1, -1)
    expect(moved.templates.map((template) => template.key)).toEqual([
      'custom-existing',
      'make-model',
    ])

    expect(moveTemplate(config, 0, -1)).toBe(config)
    expect(moveTemplate(config, 1, 1)).toBe(config)
    expect(config.templates.map((template) => template.key)).toEqual([
      'make-model',
      'custom-existing',
    ])
  })
})
