import App from './app.svelte'
import '../app.scss'

import '@ggchivalrous/db-ui/components/theme/index.css'

const target = document.getElementById('app')
if (!target) throw new Error('Application mount element is missing')

const app = new App({ target })

export default app
