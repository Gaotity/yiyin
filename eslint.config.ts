import antfu from '@antfu/eslint-config'

export default antfu(
  {
    typescript: true,
    formatters: true,
    svelte: true,
    ignores: ['electron/src/modules/logger/**/*'],
  },
  {
    rules: {
      'max-len': 'off',
      'no-console': 'off',
      'antfu/if-newline': 'off',
      'markdown/require-alt-text': 'off',
      'import/no-mutable-exports': 'off',
      'svelte/no-at-html-tags': 'off',
      'e18e/prefer-static-regex': 'off',
      'no-self-assign': 'off',
      'no-irregular-whitespace': 'off',
      'pnpm/yaml-enforce-settings': 'off',
    },
  },
)
