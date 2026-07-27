import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { App } from './app/App'
import type { PlatformClient } from './platform/client'
import './styles/index.css'

const root = document.getElementById('root')

if (!root) {
  throw new Error('Missing root element')
}

let client: PlatformClient | undefined
if (
  import.meta.env.DEV &&
  new URLSearchParams(window.location.search).get('platform') === 'fake'
) {
  const { createBrowserFakePlatformClient } = await import('./platform/fake')
  client = createBrowserFakePlatformClient()
}

createRoot(root).render(
  <StrictMode>
    <App client={client} />
  </StrictMode>,
)
