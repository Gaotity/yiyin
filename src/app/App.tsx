import { useState } from 'react'
import type { PlatformClient } from '../platform/client'
import { tauriClient } from '../platform/client'
import { Footer } from '../features/chrome/Footer'
import { TitleBar } from '../features/chrome/TitleBar'
import { FieldDrawer } from '../features/settings/FieldDrawer'
import { RenderingSettings } from '../features/settings/RenderingSettings'
import { TemplateDrawer } from '../features/settings/TemplateDrawer'
import '../features/settings/settings.css'
import { TaskWorkspace } from '../features/tasks/TaskWorkspace'
import '../features/tasks/tasks.css'
import { useAppController } from './useAppController'

interface AppProps {
  client?: PlatformClient
}

export function App({ client = tauriClient }: AppProps) {
  const controller = useAppController(client)
  const [fieldDrawerOpen, setFieldDrawerOpen] = useState(false)
  const [templateDrawerOpen, setTemplateDrawerOpen] = useState(false)

  if (controller.phase === 'loading') {
    return <main className="app-state">正在加载…</main>
  }
  if (controller.phase === 'error' || !controller.snapshot) {
    return (
      <main className="app-state" role="alert">
        <h1>壹印启动失败</h1>
        <p>{controller.error?.message ?? 'An internal error occurred.'}</p>
        <button type="button" onClick={() => void controller.retry()}>
          重试
        </button>
      </main>
    )
  }
  const snapshot = controller.snapshot

  return (
    <div className="app-shell">
      <TitleBar
        config={snapshot.config}
        resources={snapshot.resources}
        onFontChange={(font) => {
          const config = structuredClone(snapshot.config)
          config.options.font = font
          return controller.updateConfig(config)
        }}
        onRegisterFont={controller.registerFont}
        onRemoveFont={controller.removeFont}
        onReset={() => void controller.resetConfig()}
        onMinimize={() => void client.minimizeWindow()}
        onClose={() => void client.closeWindow()}
        onOpen={(destination) => void client.openExternalUrl(destination)}
      />
      <main className="app-content">
        {controller.snapshot.warnings.length > 0 && (
          <aside className="migration-warning">
            <strong>迁移提示</strong>
            {controller.snapshot.warnings.map((warning) => (
              <p key={warning}>{warning}</p>
            ))}
          </aside>
        )}
        <div className="settings-task-layout">
          <RenderingSettings
            config={controller.snapshot.config}
            onSave={controller.updateConfig}
          />
          <TaskWorkspace
            config={controller.snapshot.config}
            tasks={controller.snapshot.tasks}
            selectedId={controller.selectedTaskId}
            readExif={controller.readTaskExif}
            onSelect={controller.selectTask}
            onChooseImages={controller.chooseImages}
            onStartTasks={controller.startTasks}
            onPreviewTask={controller.previewTask}
            onInvalidatePreview={controller.invalidatePreviewRequests}
            onChooseOutput={controller.chooseOutputDirectory}
            onOpenOutput={controller.openOutputDirectory}
            onClear={controller.clearTasks}
            onOpenFields={() => setFieldDrawerOpen(true)}
            onOpenTemplates={() => setTemplateDrawerOpen(true)}
          />
        </div>
        <div className="guide-copy" aria-hidden="true">
          <strong>白嫖指南(●°u°●)​ 」</strong>
          <span>萌新指北(｡･ω･｡)</span>
        </div>
      </main>
      <FieldDrawer
        open={fieldDrawerOpen}
        config={controller.snapshot.config}
        onOpenChange={setFieldDrawerOpen}
        onSave={controller.updateConfig}
        onRegisterOverlay={controller.registerOverlay}
      />
      <TemplateDrawer
        open={templateDrawerOpen}
        config={controller.snapshot.config}
        onOpenChange={setTemplateDrawerOpen}
        onSave={controller.updateConfig}
      />
      <Footer
        onOpen={(destination) => void client.openExternalUrl(destination)}
      />
    </div>
  )
}
