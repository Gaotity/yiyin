import { resolve } from 'node:path'
import { defineConfig } from 'vitest/config'

export default defineConfig({
  resolve: {
    alias: {
      '@': resolve(import.meta.dirname),
      '@common': resolve(import.meta.dirname, 'common'),
      '@root': resolve(import.meta.dirname, 'electron'),
      '@src': resolve(import.meta.dirname, 'electron/src'),
      '@modules': resolve(import.meta.dirname, 'electron/src/modules'),
      '@utils': resolve(import.meta.dirname, 'electron/src/utils'),
    },
  },
  test: {
    environment: 'node',
    include: ['tests/**/*.test.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json-summary'],
      include: [
        'electron/security/**/*.ts',
        'electron/ipc/{schemas,sender,errors}.ts',
        'electron/resources/policy.ts',
        'electron/config/migrate.ts',
        'electron/image/blur.ts',
        'electron/src/modules/exif-reader/index.ts',
      ],
      thresholds: {
        lines: 90,
        statements: 90,
        functions: 90,
        branches: 85,
      },
    },
  },
})
