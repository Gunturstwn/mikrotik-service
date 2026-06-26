import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './assets/main.css'

// Route components
import LandingPage from './pages/LandingPage.vue'
import LoginPage from './pages/LoginPage.vue'
import DashboardPage from './pages/DashboardPage.vue'
import DashboardHome from './pages/dashboard/DashboardHome.vue'
import ProfilePage from './pages/dashboard/ProfilePage.vue'
import MikrotikPage from './pages/dashboard/MikrotikPage.vue'
import TelegramPage from './pages/dashboard/TelegramPage.vue'
import RolesPage from './pages/dashboard/RolesPage.vue'
import AuditLogsPage from './pages/dashboard/AuditLogsPage.vue'
import BackupLogsPage from './pages/dashboard/BackupLogsPage.vue'
import MetricsPage from './pages/dashboard/MetricsPage.vue'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    { path: '/', component: LandingPage },
    { path: '/login', component: LoginPage },
    {
      path: '/dashboard',
      component: DashboardPage,
      children: [
        { path: '', component: DashboardHome },
        { path: 'profile', component: ProfilePage },
        { path: 'mikrotik', component: MikrotikPage },
        { path: 'telegram', component: TelegramPage },
        { path: 'roles', component: RolesPage },
        { path: 'audit-logs', component: AuditLogsPage },
        { path: 'backup-logs', component: BackupLogsPage },
        { path: 'metrics', component: MetricsPage },
      ],
    },
  ],
  scrollBehavior(to) {
    if (to.hash) {
      return { el: to.hash, behavior: 'smooth' }
    }
    return { top: 0 }
  }
})

const app = createApp(App)
app.use(router)
app.mount('#app')
