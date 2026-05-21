<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { authStorage } from '@/api/auth'
import { getMyProfile, type UserProfileResponse } from '@/api/user'
import { listMikrotikClients, type MikrotikClientResponse } from '@/api/mikrotik'

const user = ref<UserProfileResponse | null>(null)
const devices = ref<MikrotikClientResponse[]>([])
const isLoading = ref(true)

onMounted(async () => {
  try {
    const [u, d] = await Promise.all([getMyProfile(), listMikrotikClients()])
    user.value = u
    devices.value = d
  } catch { /* handled by layout auth guard */ }
  isLoading.value = false
})
</script>

<template>
  <div class="home">
    <div v-if="isLoading" class="loading">
      <svg class="spin" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
      </svg>
    </div>
    <template v-else>
      <h1 class="page-title">Selamat Datang, {{ user?.name?.split(' ')[0] ?? 'User' }}</h1>
      <p class="page-sub">Ringkasan akun dan perangkat Anda.</p>

      <div class="stats-grid">
        <div class="stat-card glass-card">
          <div class="stat-icon blue">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="14" width="20" height="7" rx="2"/>
              <path d="M6 18h.01M10 18h.01"/>
              <path d="M12 3v4M8 7h8M7 7l5-4 5 4"/>
            </svg>
          </div>
          <div>
            <span class="stat-number">{{ devices.length }}</span>
            <span class="stat-label">Perangkat MikroTik</span>
          </div>
        </div>
        <div class="stat-card glass-card">
          <div class="stat-icon cyan">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
          </div>
          <div>
            <span class="stat-number">{{ user?.is_verified ? 'Aktif' : 'Pending' }}</span>
            <span class="stat-label">Status Akun</span>
          </div>
        </div>
        <div class="stat-card glass-card">
          <div class="stat-icon violet">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
              <circle cx="12" cy="7" r="4"/>
            </svg>
          </div>
          <div>
            <span class="stat-number">{{ user?.roles?.join(', ') || 'User' }}</span>
            <span class="stat-label">Role</span>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>

<style scoped>
.home { max-width: 900px; }
.loading {
  display: flex; justify-content: center; padding: var(--space-16);
  color: var(--color-text-secondary);
}
.page-title {
  font-size: var(--font-size-3xl);
  font-weight: 800;
  margin-bottom: var(--space-2);
}
.page-sub {
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-8);
}
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}
.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-6);
}
.stat-icon {
  width: 48px; height: 48px;
  display: flex; align-items: center; justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}
.stat-icon.blue { background: rgba(59,130,246,0.12); color: var(--color-blue); border: 1px solid rgba(59,130,246,0.2); }
.stat-icon.cyan { background: rgba(6,182,212,0.12); color: var(--color-cyan); border: 1px solid rgba(6,182,212,0.2); }
.stat-icon.violet { background: rgba(139,92,246,0.12); color: var(--color-violet); border: 1px solid rgba(139,92,246,0.2); }
.stat-number {
  display: block;
  font-size: var(--font-size-xl);
  font-weight: 800;
  color: var(--color-text-primary);
}
.stat-label {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }
@media (max-width: 768px) { .stats-grid { grid-template-columns: 1fr; } }
</style>
