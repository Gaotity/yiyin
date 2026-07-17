import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'
import { configDefaults } from 'vitest/config'

export default defineConfig({
  clearScreen: false,
  plugins: [react()],
  optimizeDeps: {
    entries: ['index.html'],
  },
  server: {
    strictPort: true,
    port: 1420,
  },
  test: {
    environment: 'jsdom',
    exclude: [...configDefaults.exclude, 'tests/ui/**'],
    passWithNoTests: true,
  },
})
