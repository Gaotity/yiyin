export const ipcChannels = {
  app: { minimize: 'platform:app:minimize', close: 'platform:app:close' },
  config: {
    get: 'platform:config:get',
    update: 'platform:config:update',
    reset: 'platform:config:reset',
    chooseOutputDirectory: 'platform:config:choose-output-directory',
    openOutputDirectory: 'platform:config:open-output-directory',
  },
  files: {
    registerImages: 'platform:files:register-images',
    registerFont: 'platform:files:register-font',
    removeFont: 'platform:files:remove-font',
    registerOverlay: 'platform:files:register-overlay',
  },
  tasks: {
    start: 'platform:tasks:start',
    preview: 'platform:tasks:preview',
    readExif: 'platform:tasks:read-exif',
    clear: 'platform:tasks:clear',
    completeTextRender: 'platform:tasks:complete-text-render',
    completeShadowRender: 'platform:tasks:complete-shadow-render',
  },
  events: {
    progress: 'platform:event:progress',
    failure: 'platform:event:failure',
    textRender: 'platform:event:text-render',
    shadowRender: 'platform:event:shadow-render',
  },
} as const
