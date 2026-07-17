import type { PlatformClient } from '../platform/client'
import { tauriClient } from '../platform/client'
import type { TaskStateDto } from '../platform/types'
import { Footer } from '../features/chrome/Footer'
import { TitleBar } from '../features/chrome/TitleBar'
import { useAppController } from './useAppController'

interface AppProps {
  client?: PlatformClient
}

const STATE_LABELS: Record<TaskStateDto, string> = {
  registered: '等待处理',
  queued: '等待处理',
  running: '处理中',
  completed: '已完成',
  failed: '处理失败',
  cancelled: '已取消',
}

export function App({ client = tauriClient }: AppProps) {
  const controller = useAppController(client)

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

  return (
    <div className="app-shell">
      <TitleBar
        config={controller.snapshot.config}
        resources={controller.snapshot.resources}
        onFontChange={(font) => {
          const config = structuredClone(controller.snapshot?.config)
          if (config) {
            config.options.font = font
            void controller.updateConfig(config)
          }
        }}
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
        <section className="task-list" aria-label="图片任务">
          {controller.snapshot.tasks.map((task) => (
            <button
              className={
                task.id === controller.selectedTaskId ? 'task selected' : 'task'
              }
              key={task.id}
              type="button"
              onClick={() => controller.selectTask(task.id)}
            >
              <span>{task.displayName}</span>
              <span>{STATE_LABELS[task.state]}</span>
              <span>{task.progress}%</span>
            </button>
          ))}
        </section>
        <div className="guide-copy" aria-hidden="true">
          <strong>白嫖指南(●°u°●)​ 」</strong>
          <span>萌新指北(｡･ω･｡)</span>
        </div>
        <div className="primary-actions">
          <button type="button" onClick={() => void controller.chooseImages()}>
            添加图片
          </button>
          <button type="button" onClick={() => void controller.startTasks()}>
            生成印框
          </button>
          <button type="button">参数设置</button>
          <button type="button">模板设置</button>
        </div>
      </main>
      <Footer
        onOpen={(destination) => void client.openExternalUrl(destination)}
      />
    </div>
  )
}
