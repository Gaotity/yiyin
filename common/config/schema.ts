import { z } from 'zod'

const fontSettings = z.object({
  use: z.boolean().optional(),
  bold: z.boolean().optional(),
  italic: z.boolean().optional(),
  size: z.number().min(0).max(100),
  font: z.string().max(100),
  caseType: z.enum(['lowcase', 'upcase', 'default']).optional(),
  color: z.string().max(32).optional(),
}).strict()

const field = z.object({
  use: z.boolean().optional(),
  forceUse: z.boolean().optional(),
  show: z.boolean().optional(),
  key: z.string().max(100),
  name: z.string().max(100),
  value: z.union([z.string().max(500), z.number(), z.boolean()]),
  wImg: z.string().max(500),
  bImg: z.string().max(500),
  type: z.enum(['text', 'img']),
  font: fontSettings,
}).strict()

const template = z.object({
  key: z.string().max(100),
  name: z.string().max(100),
  temp: z.string().max(1_000),
  use: z.boolean(),
  type: z.enum(['system', 'custom']),
  height: z.number().positive().max(20_000).optional(),
  verticalAlign: z.enum(['center', 'baseline']),
  font: fontSettings.required({ bold: true, italic: true, size: true, font: true }).extend({
    caseType: z.enum(['lowcase', 'upcase', 'default']),
    color: z.string().max(32),
  }),
  position: z.object({
    top: z.number().nullable(),
    bottom: z.number().nullable(),
    left: z.number().nullable(),
    right: z.number().nullable(),
  }).strict().optional(),
}).strict()

export const outputOptionsSchema = z.object({
  iot: z.boolean(),
  landscape: z.boolean(),
  solid_bg: z.boolean(),
  solid_color: z.string().regex(/^#[\da-f]{3,8}$/i),
  bg_rate: z.object({ w: z.number().min(0).max(100), h: z.number().min(0).max(100) }).strict(),
  bg_rate_show: z.boolean(),
  origin_wh_output: z.boolean(),
  radius: z.number().min(0).max(50),
  radius_show: z.boolean(),
  shadow: z.number().min(0).max(50),
  shadow_show: z.boolean(),
  font: z.string().max(100),
  main_img_w_rate: z.number().min(1).max(100),
  text_margin: z.number().min(0).max(100),
  quality: z.number().int().min(1).max(100),
  mini_top_bottom_margin: z.number().min(0).max(100),
  bg_blur: z.number().min(0).max(100),
  preview_show: z.boolean(),
}).strict()

export const configMutationSchema = z.object({
  options: outputOptionsSchema,
  tempFields: z.array(field).max(100),
  customTempFields: z.array(field).max(100),
  temps: z.array(template).max(100),
}).strict()
