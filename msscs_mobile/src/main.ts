import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './styles/mobile.css'

// Views
import FilesView from './views/FilesView.vue'
import NodesView from './views/NodesView.vue'
import SyncView from './views/SyncView.vue'
import SettingsView from './views/SettingsView.vue'
import FileViewerView from './views/FileViewerView.vue'
import SecurityInfoView from './views/SecurityInfoView.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', redirect: '/files' },
    { path: '/files', component: FilesView },
    { path: '/nodes', component: NodesView },
    { path: '/security', component: SecurityInfoView },
    { path: '/sync', component: SyncView },
    { path: '/settings', component: SettingsView },
    { path: '/viewer/:path', component: FileViewerView, props: true },
  ],
})

const pinia = createPinia()
const app = createApp(App)

app.use(pinia)
app.use(router)
app.mount('#app')
