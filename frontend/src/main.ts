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
