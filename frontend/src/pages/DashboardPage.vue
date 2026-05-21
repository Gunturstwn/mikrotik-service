<script setup lang="ts">
import { onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { authStorage, logoutApi } from '@/api/auth'
import DashboardSidebar from '@/components/DashboardSidebar.vue'

const router = useRouter()

const handleLogout = async () => {
  try { await logoutApi() } catch { /* ignore */ }
  authStorage.clear()
  router.push('/')
}

onMounted(() => {
  if (!authStorage.isLoggedIn()) {
    router.push('/login')
  }
})
</script>

<template>
  <div class="dashboard-root">
    <!-- Background -->
    <div class="dashboard-bg" aria-hidden="true">
      <div class="bg-orb orb-1"></div>
      <div class="bg-orb orb-2"></div>
      <div class="bg-grid"></div>
    </div>

    <!-- Top Bar -->
    <header class="dashboard-header">
      <div class="container-full header-inner">
        <router-link to="/" class="header-logo">
          <svg width="28" height="28" viewBox="0 0 32 32" fill="none">
            <circle cx="16" cy="16" r="16" fill="url(#dhLg1)"/>
            <path d="M8 16 L16 8 L24 16 L16 24 Z" fill="white" opacity="0.9"/>
            <circle cx="16" cy="16" r="4" fill="url(#dhLg2)"/>
            <defs>
              <linearGradient id="dhLg1" x1="0" y1="0" x2="32" y2="32">
                <stop offset="0%" stop-color="#06b6d4"/>
                <stop offset="100%" stop-color="#3b82f6"/>
              </linearGradient>
              <linearGradient id="dhLg2" x1="0" y1="0" x2="32" y2="32">
                <stop offset="0%" stop-color="#22d3ee"/>
                <stop offset="100%" stop-color="#60a5fa"/>
              </linearGradient>
            </defs>
          </svg>
          <span class="header-logo-text">My PC24</span>
        </router-link>

        <button class="btn btn-secondary logout-btn" @click="handleLogout">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
            <polyline points="16 17 21 12 16 7"/>
            <line x1="21" y1="12" x2="9" y2="12"/>
          </svg>
          Keluar
        </button>
      </div>
    </header>

    <!-- Body: Sidebar + Content -->
    <div class="dashboard-body">
      <DashboardSidebar />
      <main class="dashboard-content">
        <router-view />
      </main>
    </div>
  </div>
</template>

<style scoped>
.dashboard-root {
  min-height: 100vh;
  background: var(--color-bg);
  position: relative;
  display: flex;
  flex-direction: column;
}

/* Background */
.dashboard-bg { position: fixed; inset: 0; pointer-events: none; z-index: 0; }
.bg-orb { position: absolute; border-radius: 50%; filter: blur(80px); }
.orb-1 {
  width: 500px; height: 500px;
  background: radial-gradient(circle, rgba(6,182,212,0.08) 0%, transparent 70%);
  top: -200px; right: -100px;
}
.orb-2 {
  width: 400px; height: 400px;
  background: radial-gradient(circle, rgba(59,130,246,0.06) 0%, transparent 70%);
  bottom: -100px; left: -100px;
}
.bg-grid {
  position: absolute; inset: 0;
  background-image:
    linear-gradient(rgba(6,182,212,0.02) 1px, transparent 1px),
    linear-gradient(90deg, rgba(6,182,212,0.02) 1px, transparent 1px);
  background-size: 60px 60px;
  mask-image: radial-gradient(ellipse 80% 80% at 50% 50%, black, transparent);
}

/* Header */
.dashboard-header {
  position: sticky;
  top: 0;
  z-index: 100;
  background: rgba(6, 11, 20, 0.9);
  backdrop-filter: blur(20px);
  border-bottom: 1px solid var(--color-border);
}
.container-full {
  width: 100%;
  padding: 0 var(--space-6);
}
.header-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 64px;
}
.header-logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.header-logo-text {
  font-size: var(--font-size-lg);
  font-weight: 800;
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}
.logout-btn {
  padding: var(--space-2) var(--space-4);
  font-size: var(--font-size-sm);
}

/* Body */
.dashboard-body {
  display: flex;
  flex: 1;
  position: relative;
  z-index: 1;
}
.dashboard-content {
  flex: 1;
  padding: var(--space-8);
  overflow-y: auto;
  min-height: calc(100vh - 64px);
}

@media (max-width: 768px) {
  .dashboard-body { flex-direction: column; }
  .dashboard-content { padding: var(--space-4); }
}
</style>
