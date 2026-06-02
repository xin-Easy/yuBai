import { createApp } from 'vue'
import { createPinia } from 'pinia'
import ElementPlus from 'element-plus'
import '@unocss/reset/tailwind.css'
import 'element-plus/dist/index.css'
import 'virtual:uno.css'
import './styles/index.scss'
import App from './App.vue'
import { router } from './router'

const app = createApp(App)
app.use(createPinia())
app.use(ElementPlus)
app.use(router)
app.mount('#app')
