import { createApp } from 'vue'
import App from './App.vue'
import './styles/theme.css'
import './styles/vue-extras.css'

// 防止 Tailwind preflight 覆盖 Naive UI 样式
const meta = document.createElement('meta')
meta.name = 'naive-ui-style'
document.head.appendChild(meta)

createApp(App).mount('#app')
