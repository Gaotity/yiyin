function isRecord(value: unknown): value is Record<string, unknown> {
  return Object.prototype.toString.call(value) === '[object Object]'
}

export function normalize<T extends object>(origin: unknown, model: T): T {
  const source = isRecord(origin) ? structuredClone(origin) : {}
  const result: Record<string, unknown> = {}

  for (const [key, modelValue] of Object.entries(model)) {
    const sourceValue = source[key]
    if (isRecord(modelValue) && Object.keys(modelValue).length > 0) {
      result[key] = normalize(sourceValue, modelValue)
    }
    else if (Array.isArray(modelValue) && isRecord(modelValue[0])) {
      result[key] = Array.isArray(sourceValue)
        ? sourceValue.filter(isRecord).map(item => normalize(item, modelValue[0]))
        : []
    }
    else {
      result[key] = sourceValue === undefined ? modelValue : sourceValue
    }
  }

  return result as T
}

export const cpObj = <T>(obj: T): T => structuredClone(obj)
