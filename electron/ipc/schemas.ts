import { configMutationSchema, outputOptionsSchema } from '@common/config/schema'
import { z } from 'zod'

const resourceId = z.string().regex(/^[a-f0-9]{32}$/)
const dataUrl = z.string().startsWith('data:image/png;base64,').max(20 * 1024 * 1024)
const fileCandidate = z.object({ path: z.string().min(1).max(4_096), name: z.string().min(1).max(255) }).strict()

export const ipcSchemas = {
  empty: z.undefined(),
  taskId: z.object({ taskId: resourceId }).strict(),
  outputOptions: outputOptionsSchema,
  configMutation: configMutationSchema,
  imageFiles: z.object({ files: z.array(fileCandidate).min(1).max(100) }).strict(),
  font: z.object({ file: fileCandidate, name: z.string().trim().min(1).max(100) }).strict(),
  removeFont: z.object({ name: z.string().trim().min(1).max(100) }).strict(),
  overlay: z.object({ file: fileCandidate, slot: z.string().regex(/^[\w-]{1,100}$/) }).strict(),
  textRender: z.object({ taskId: resourceId, images: z.array(z.object({ data: dataUrl, w: z.number().positive().optional(), h: z.number().positive().optional() }).strict()).max(20) }).strict(),
  shadowRender: z.object({ taskId: resourceId, dataUrl }).strict(),
}
