/**
 * Named config edit intents — the only places the UI computes the next
 * config. Each intent is a pure function: it never mutates its input and
 * returns the config to persist. When an edit does not apply (a protected
 * system template, an out-of-bounds move), the intent returns the input
 * unchanged so the caller can skip saving.
 */
import type {
  NumericOptionDto,
  PublicConfigDto,
  TemplateDto,
  TemplateFieldDto,
} from '../../platform/types'

export function setNumberOption(
  config: PublicConfigDto,
  key: NumericOptionDto,
  value: number,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options[key] = value
  return next
}

export function setRadiusVisible(
  config: PublicConfigDto,
  visible: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.radiusVisible = visible
  return next
}

export function setShadowVisible(
  config: PublicConfigDto,
  visible: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.shadowVisible = visible
  return next
}

export function setBackgroundRatioVisible(
  config: PublicConfigDto,
  visible: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.backgroundRatioVisible = visible
  if (visible) {
    // Mirrors the domain coupling (ADR 0005) so the UI never displays a
    // contradictory state; the domain enforces it on every write path.
    next.options.landscape = false
  }
  return next
}

export function setBackgroundRatioWidth(
  config: PublicConfigDto,
  width: number,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.backgroundRatio.width = width
  return next
}

export function setBackgroundRatioHeight(
  config: PublicConfigDto,
  height: number,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.backgroundRatio.height = height
  return next
}

export function swapBackgroundRatio(config: PublicConfigDto): PublicConfigDto {
  const next = structuredClone(config)
  const { width } = next.options.backgroundRatio
  next.options.backgroundRatio.width = next.options.backgroundRatio.height
  next.options.backgroundRatio.height = width
  return next
}

export function setSolidBackground(
  config: PublicConfigDto,
  enabled: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.solidBackground = enabled
  return next
}

export function setSolidColor(
  config: PublicConfigDto,
  color: string,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.solidColor = color
  return next
}

export function setLandscape(
  config: PublicConfigDto,
  landscape: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.landscape = landscape
  return next
}

export function setQuickOutput(
  config: PublicConfigDto,
  enabled: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.quickOutput = enabled
  return next
}

export function setPreviewVisible(
  config: PublicConfigDto,
  visible: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.previewVisible = visible
  return next
}

export function setFont(
  config: PublicConfigDto,
  font: string,
): PublicConfigDto {
  const next = structuredClone(config)
  next.options.font = font
  return next
}

export function upsertTemplateField(
  config: PublicConfigDto,
  field: TemplateFieldDto,
  custom: boolean,
): PublicConfigDto {
  const next = structuredClone(config)
  const list = custom ? next.customTemplateFields : next.templateFields
  const index = list.findIndex((candidate) => candidate.key === field.key)
  if (index === -1) {
    list.push(field)
  } else {
    list[index] = field
  }
  return next
}

export function removeCustomTemplateField(
  config: PublicConfigDto,
  key: string,
): PublicConfigDto {
  if (!config.customTemplateFields.some((field) => field.key === key)) {
    return config
  }
  const next = structuredClone(config)
  next.customTemplateFields = next.customTemplateFields.filter(
    (field) => field.key !== key,
  )
  return next
}

export function upsertTemplate(
  config: PublicConfigDto,
  template: TemplateDto,
): PublicConfigDto {
  const next = structuredClone(config)
  const index = next.templates.findIndex(
    (candidate) => candidate.key === template.key,
  )
  if (index === -1) {
    next.templates.push(template)
  } else {
    next.templates[index] = template
  }
  return next
}

export function removeTemplate(
  config: PublicConfigDto,
  template: TemplateDto,
): PublicConfigDto {
  const stored = config.templates.find(
    (candidate) => candidate.key === template.key,
  )
  if (!stored || stored.kind === 'system') {
    return config
  }
  const next = structuredClone(config)
  next.templates = next.templates.filter(
    (candidate) => candidate.key !== template.key,
  )
  return next
}

export function moveTemplate(
  config: PublicConfigDto,
  index: number,
  offset: -1 | 1,
): PublicConfigDto {
  const destination = index + offset
  if (destination < 0 || destination >= config.templates.length) {
    return config
  }
  const next = structuredClone(config)
  const current = next.templates[index]
  const adjacent = next.templates[destination]
  if (!current || !adjacent) {
    return config
  }
  next.templates[index] = adjacent
  next.templates[destination] = current
  return next
}
