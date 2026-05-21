<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()

const navItems = [
  { label: 'Dashboard', icon: 'home', to: '/dashboard' },
  { label: 'Profil', icon: 'user', to: '/dashboard/profile' },
  { label: 'MikroTik', icon: 'router', to: '/dashboard/mikrotik' },
  { label: 'Telegram', icon: 'telegram', to: '/dashboard/telegram' },
]

const isActive = (path: string) => {
  if (path === '/dashboard') return route.path === '/dashboard'
  return route.path.startsWith(path)
}
</script>

<template>
  <aside class="sidebar">
    <nav class="sidebar-nav">
      <router-link
        v-for="item in navItems"
        :key="item.to"
        :to="item.to"
        class="nav-item"
        :class="{ active: isActive(item.to) }"
      >
        <!-- Home icon -->
        <svg v-if="item.icon === 'home'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
          <polyline points="9 22 9 12 15 12 15 22"/>
        </svg>
        <!-- User icon -->
        <svg v-else-if="item.icon === 'user'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
          <circle cx="12" cy="7" r="4"/>
        </svg>
        <!-- Router icon -->
        <svg v-else-if="item.icon === 'router'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <rect x="2" y="14" width="20" height="7" rx="2"/>
          <path d="M6 18h.01M10 18h.01"/>
          <path d="M12 3v4M8 7h8M7 7l5-4 5 4"/>
        </svg>
        <!-- Telegram icon -->
        <svg v-else-if="item.icon === 'telegram'" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/>
        </svg>
        <span>{{ item.label }}</span>
      </router-link>
    </nav>
  </aside>
</template>

<style scoped>
.sidebar {
  width: 240px;
  min-height: calc(100vh - 64px);
  background: rgba(13, 24, 41, 0.6);
  border-right: 1px solid var(--color-border);
  padding: var(--space-4) var(--space-3);
  position: sticky;
  top: 64px;
  flex-shrink: 0;
}

.sidebar-nav {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-4);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  transition: all var(--transition-base);
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--color-text-primary);
}

.nav-item.active {
  background: rgba(6, 182, 212, 0.1);
  color: var(--color-cyan);
  border: 1px solid rgba(6, 182, 212, 0.2);
}

.nav-item.active svg {
  color: var(--color-cyan);
}

@media (max-width: 768px) {
  .sidebar {
    width: 100%;
    min-height: auto;
    position: static;
    flex-direction: row;
    border-right: none;
    border-bottom: 1px solid var(--color-border);
    padding: var(--space-2) var(--space-3);
  }
  .sidebar-nav {
    flex-direction: row;
    overflow-x: auto;
    gap: var(--space-2);
  }
  .nav-item {
    white-space: nowrap;
    padding: var(--space-2) var(--space-3);
  }
}
</style>
