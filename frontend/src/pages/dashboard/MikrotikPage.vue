<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  listMikrotikClients,
  createMikrotikClient,
  deleteMikrotikClient,
  getMikrotikResource,
  testMikrotikConnection,
  type MikrotikClientResponse,
  type MikrotikClientRequest,
  type MikrotikResourceResponse,
} from '@/api/mikrotik'

const devices = ref<MikrotikClientResponse[]>([])
const isLoading = ref(true)
const error = ref('')

// Add device modal
const showAddModal = ref(false)
const addForm = ref<MikrotikClientRequest>({
  name_device: '', host: '', username: '', password: '',
  port_winbox: '', port_api: '', port_ftp: '', port_ssh: '',
  location: '',
})
const isAdding = ref(false)
const addError = ref('')

// Device detail / resource
const selectedDevice = ref<MikrotikClientResponse | null>(null)
const deviceResource = ref<MikrotikResourceResponse | null>(null)
const isLoadingResource = ref(false)
const connectionStatus = ref<Record<string, boolean | null>>({})

const fetchDevices = async () => {
  isLoading.value = true; error.value = ''
  try {
    devices.value = await listMikrotikClients()
    // Auto-test koneksi semua device setelah list dimuat
    checkAllConnections()
  }
  catch (e: any) { error.value = e.message ?? 'Gagal memuat perangkat.' }
  finally { isLoading.value = false }
}

/** Test koneksi semua device secara paralel */
const checkAllConnections = () => {
  devices.value.forEach(device => {
    connectionStatus.value[device.id] = null // loading state
    testMikrotikConnection(device.id)
      .then(ok => { connectionStatus.value[device.id] = ok })
      .catch(() => { connectionStatus.value[device.id] = false })
  })
}

const handleAdd = async () => {
  isAdding.value = true; addError.value = ''
  try {
    const created = await createMikrotikClient(addForm.value)
    devices.value.unshift(created)
    showAddModal.value = false
    addForm.value = { name_device: '', host: '', username: '', password: '', port_winbox: '', port_api: '', port_ftp: '', port_ssh: '', location: '' }
  } catch (e: any) { addError.value = e.message ?? 'Gagal menambah perangkat.' }
  finally { isAdding.value = false }
}

const handleDelete = async (id: string) => {
  if (!confirm('Hapus perangkat ini?')) return
  try {
    await deleteMikrotikClient(id)
    devices.value = devices.value.filter(d => d.id !== id)
    if (selectedDevice.value?.id === id) selectedDevice.value = null
  } catch (e: any) { alert(e.message ?? 'Gagal menghapus.') }
}

const selectDevice = async (device: MikrotikClientResponse) => {
  selectedDevice.value = device
  deviceResource.value = null
  isLoadingResource.value = true
  try { deviceResource.value = await getMikrotikResource(device.id) }
  catch { /* device might be offline */ }
  finally { isLoadingResource.value = false }
}

const checkConnection = async (id: string) => {
  connectionStatus.value[id] = null
  try {
    const ok = await testMikrotikConnection(id)
    connectionStatus.value[id] = ok
  } catch { connectionStatus.value[id] = false }
}

const formatBytes = (bytes: number) => {
  if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(0)} MB`
  return `${(bytes / 1024).toFixed(0)} KB`
}

onMounted(fetchDevices)
</script>

<template>
  <div class="mikrotik-page">
    <div class="page-header">
      <div>
        <h1 class="page-title">Perangkat MikroTik</h1>
        <p class="page-sub">Kelola dan monitor perangkat router MikroTik Anda.</p>
      </div>
      <button class="btn btn-primary" @click="showAddModal = true">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        Tambah Perangkat
      </button>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="loading">
      <svg class="spin" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="error-box glass-card">
      <p>{{ error }}</p>
      <button class="btn btn-primary" @click="fetchDevices">Coba Lagi</button>
    </div>

    <!-- Content -->
    <div v-else class="content-grid">
      <!-- Device List -->
      <div class="device-list">
        <div v-if="devices.length === 0" class="empty-state glass-card">
          <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <rect x="2" y="14" width="20" height="7" rx="2"/><path d="M6 18h.01M10 18h.01"/><path d="M12 3v4M8 7h8"/>
          </svg>
          <p>Belum ada perangkat. Tambahkan perangkat MikroTik pertama Anda.</p>
        </div>

        <div
          v-for="device in devices"
          :key="device.id"
          class="device-card glass-card"
          :class="{ active: selectedDevice?.id === device.id }"
          @click="selectDevice(device)"
        >
          <div class="device-header">
            <div class="device-icon">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="14" width="20" height="7" rx="2"/><path d="M6 18h.01M10 18h.01"/><path d="M12 3v4M8 7h8M7 7l5-4 5 4"/>
              </svg>
            </div>
            <div class="device-info">
              <span class="device-name">{{ device.name_device }}</span>
              <span class="device-host">{{ device.host }}</span>
            </div>
            <div class="device-status">
              <span v-if="connectionStatus[device.id] === null" class="status-checking">
                <svg class="spin" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
              </span>
              <span v-else-if="connectionStatus[device.id] === true" class="status-badge online">Online</span>
              <span v-else-if="connectionStatus[device.id] === false" class="status-badge offline">Offline</span>
            </div>
          </div>
          <div class="device-meta">
            <span v-if="device.location" class="meta-item">📍 {{ device.location }}</span>
            <span class="meta-item">SSH: {{ device.port_ssh || '—' }}</span>
          </div>
          <div class="device-actions" @click.stop>
            <button class="action-btn" @click="checkConnection(device.id)" title="Test koneksi">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><polyline points="22 4 12 14.01 9 11.01"/>
              </svg>
            </button>
            <button class="action-btn danger" @click="handleDelete(device.id)" title="Hapus">
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
              </svg>
            </button>
          </div>
        </div>
      </div>

      <!-- Detail Panel -->
      <div v-if="selectedDevice" class="detail-panel glass-card">
        <h3 class="detail-title">{{ selectedDevice.name_device }}</h3>
        <p class="detail-host">{{ selectedDevice.host }}</p>

        <!-- Resource Info -->
        <div v-if="isLoadingResource" class="loading-sm">
          <svg class="spin" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
          <span>Memuat resource...</span>
        </div>
        <div v-else-if="deviceResource" class="resource-grid">
          <div class="resource-item">
            <span class="res-label">Uptime</span>
            <span class="res-value">{{ deviceResource.uptime }}</span>
          </div>
          <div class="resource-item">
            <span class="res-label">CPU</span>
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: deviceResource.cpu_load + '%' }" :class="{ high: deviceResource.cpu_load > 80 }"></div>
            </div>
            <span class="res-value">{{ deviceResource.cpu_load }}%</span>
          </div>
          <div class="resource-item">
            <span class="res-label">Memory</span>
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: ((deviceResource.total_memory - deviceResource.free_memory) / deviceResource.total_memory * 100) + '%' }"></div>
            </div>
            <span class="res-value">{{ formatBytes(deviceResource.total_memory - deviceResource.free_memory) }} / {{ formatBytes(deviceResource.total_memory) }}</span>
          </div>
          <div class="resource-item">
            <span class="res-label">Disk</span>
            <div class="progress-bar">
              <div class="progress-fill" :style="{ width: ((deviceResource.total_hdd_space - deviceResource.free_hdd_space) / deviceResource.total_hdd_space * 100) + '%' }"></div>
            </div>
            <span class="res-value">{{ formatBytes(deviceResource.total_hdd_space - deviceResource.free_hdd_space) }} / {{ formatBytes(deviceResource.total_hdd_space) }}</span>
          </div>
        </div>
        <div v-else class="no-resource">
          <p>Tidak dapat terhubung ke perangkat atau perangkat offline.</p>
        </div>

        <!-- Device Details -->
        <div class="device-details">
          <div class="dd-row"><span class="dd-label">Lokasi</span><span class="dd-val">{{ selectedDevice.location || '—' }}</span></div>
          <div class="dd-row"><span class="dd-label">Port Winbox</span><span class="dd-val">{{ selectedDevice.port_winbox || '—' }}</span></div>
          <div class="dd-row"><span class="dd-label">Port API</span><span class="dd-val">{{ selectedDevice.port_api || '—' }}</span></div>
          <div class="dd-row"><span class="dd-label">Port SSH</span><span class="dd-val">{{ selectedDevice.port_ssh || '—' }}</span></div>
          <div class="dd-row"><span class="dd-label">Port FTP</span><span class="dd-val">{{ selectedDevice.port_ftp || '—' }}</span></div>
          <div class="dd-row"><span class="dd-label">Timezone</span><span class="dd-val">{{ selectedDevice.timezone || '—' }}</span></div>
          <div class="dd-row"><span class="dd-label">Dibuat</span><span class="dd-val">{{ new Date(selectedDevice.created_at).toLocaleDateString('id-ID') }}</span></div>
        </div>
      </div>
    </div>

    <!-- Add Device Modal -->
    <Teleport to="body">
      <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
        <div class="modal glass-card">
          <h2 class="modal-title">Tambah Perangkat MikroTik</h2>
          <form @submit.prevent="handleAdd">
            <div class="form-row">
              <div class="form-group"><label class="form-label">Nama Perangkat *</label><input v-model="addForm.name_device" class="form-input" required placeholder="Core Router HQ" /></div>
              <div class="form-group"><label class="form-label">Host / IP *</label><input v-model="addForm.host" class="form-input" required placeholder="192.168.1.1" /></div>
            </div>
            <div class="form-row">
              <div class="form-group"><label class="form-label">Username *</label><input v-model="addForm.username" class="form-input" required placeholder="admin" /></div>
              <div class="form-group"><label class="form-label">Password *</label><input v-model="addForm.password" type="password" class="form-input" required placeholder="••••••" /></div>
            </div>
            <div class="form-row">
              <div class="form-group"><label class="form-label">Port SSH</label><input v-model="addForm.port_ssh" class="form-input" placeholder="22" /></div>
              <div class="form-group"><label class="form-label">Port Winbox</label><input v-model="addForm.port_winbox" class="form-input" placeholder="8291" /></div>
            </div>
            <div class="form-row">
              <div class="form-group"><label class="form-label">Port API</label><input v-model="addForm.port_api" class="form-input" placeholder="8728" /></div>
              <div class="form-group"><label class="form-label">Port FTP</label><input v-model="addForm.port_ftp" class="form-input" placeholder="21" /></div>
            </div>
            <div class="form-group"><label class="form-label">Lokasi</label><input v-model="addForm.location" class="form-input" placeholder="Jakarta Data Center, Rack A1" /></div>

            <div v-if="addError" class="alert-error">{{ addError }}</div>

            <div class="modal-actions">
              <button type="submit" class="btn btn-primary" :disabled="isAdding">{{ isAdding ? 'Menyimpan...' : 'Simpan' }}</button>
              <button type="button" class="btn btn-secondary" @click="showAddModal = false">Batal</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.mikrotik-page { width: 100%; }
.page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: var(--space-6); flex-wrap: wrap; gap: var(--space-4); }
.page-title { font-size: var(--font-size-2xl); font-weight: 800; margin-bottom: var(--space-1); }
.page-sub { font-size: var(--font-size-sm); color: var(--color-text-secondary); }
.loading { display: flex; justify-content: center; padding: var(--space-16); }
.error-box { padding: var(--space-8); text-align: center; }
.empty-state { padding: var(--space-10); display: flex; flex-direction: column; align-items: center; gap: var(--space-4); color: var(--color-text-muted); text-align: center; }

.content-grid { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-6); align-items: start; }
.device-list { display: flex; flex-direction: column; gap: var(--space-3); }

/* Device Card */
.device-card { padding: var(--space-4) var(--space-5); cursor: pointer; transition: all var(--transition-base); }
.device-card.active { border-color: var(--color-cyan); background: rgba(6,182,212,0.06); }
.device-header { display: flex; align-items: center; gap: var(--space-3); margin-bottom: var(--space-2); }
.device-icon { width: 36px; height: 36px; display: flex; align-items: center; justify-content: center; background: rgba(6,182,212,0.1); border: 1px solid rgba(6,182,212,0.2); border-radius: var(--radius-md); color: var(--color-cyan); flex-shrink: 0; }
.device-info { flex: 1; min-width: 0; }
.device-name { display: block; font-size: var(--font-size-sm); font-weight: 700; color: var(--color-text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.device-host { font-size: var(--font-size-xs); color: var(--color-text-muted); }
.device-status { flex-shrink: 0; }
.status-badge { font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: var(--radius-full); letter-spacing: 0.03em; }
.status-badge.online { background: rgba(16,185,129,0.15); color: var(--color-emerald); border: 1px solid rgba(16,185,129,0.3); }
.status-badge.offline { background: rgba(239,68,68,0.12); color: #f87171; border: 1px solid rgba(239,68,68,0.25); }
.status-checking { color: var(--color-text-muted); display: flex; align-items: center; }
.device-meta { display: flex; gap: var(--space-3); margin-bottom: var(--space-2); }
.meta-item { font-size: var(--font-size-xs); color: var(--color-text-muted); }
.device-actions { display: flex; gap: var(--space-2); }
.action-btn { width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; border-radius: var(--radius-sm); background: rgba(255,255,255,0.05); border: 1px solid var(--color-border); color: var(--color-text-secondary); transition: all var(--transition-base); cursor: pointer; }
.action-btn:hover { background: rgba(6,182,212,0.1); border-color: rgba(6,182,212,0.3); color: var(--color-cyan); }
.action-btn.danger:hover { background: rgba(239,68,68,0.1); border-color: rgba(239,68,68,0.3); color: #f87171; }

/* Detail Panel */
.detail-panel { padding: var(--space-6); position: sticky; top: 80px; }
.detail-title { font-size: var(--font-size-lg); font-weight: 700; margin-bottom: var(--space-1); }
.detail-host { font-size: var(--font-size-sm); color: var(--color-text-muted); margin-bottom: var(--space-5); }
.loading-sm { display: flex; align-items: center; gap: var(--space-2); font-size: var(--font-size-sm); color: var(--color-text-secondary); margin-bottom: var(--space-4); }
.no-resource { font-size: var(--font-size-sm); color: var(--color-text-muted); padding: var(--space-4); background: rgba(239,68,68,0.05); border: 1px solid rgba(239,68,68,0.15); border-radius: var(--radius-md); margin-bottom: var(--space-4); }

/* Resource Grid */
.resource-grid { display: flex; flex-direction: column; gap: var(--space-4); margin-bottom: var(--space-6); padding-bottom: var(--space-5); border-bottom: 1px solid var(--color-border); }
.resource-item { display: flex; flex-direction: column; gap: var(--space-1); }
.res-label { font-size: var(--font-size-xs); font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.res-value { font-size: var(--font-size-sm); color: var(--color-text-primary); font-weight: 600; }
.progress-bar { height: 6px; background: rgba(255,255,255,0.06); border-radius: 3px; overflow: hidden; }
.progress-fill { height: 100%; background: var(--gradient-primary); border-radius: 3px; transition: width 0.5s ease; }
.progress-fill.high { background: linear-gradient(135deg, #f59e0b, #ef4444); }

/* Device Details */
.device-details { display: flex; flex-direction: column; gap: var(--space-2); }
.dd-row { display: flex; justify-content: space-between; padding: var(--space-2) 0; border-bottom: 1px solid rgba(255,255,255,0.03); }
.dd-label { font-size: var(--font-size-xs); color: var(--color-text-muted); }
.dd-val { font-size: var(--font-size-xs); color: var(--color-text-primary); font-weight: 500; }

/* Modal */
.modal-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; padding: var(--space-4); }
.modal { width: 100%; max-width: 560px; padding: var(--space-8); max-height: 90vh; overflow-y: auto; }
.modal-title { font-size: var(--font-size-xl); font-weight: 800; margin-bottom: var(--space-6); }
.form-row { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-4); }
.form-group { margin-bottom: var(--space-4); }
.alert-error { padding: var(--space-3) var(--space-4); background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.25); border-radius: var(--radius-md); color: #fca5a5; font-size: var(--font-size-sm); margin-bottom: var(--space-4); }
.modal-actions { display: flex; gap: var(--space-3); margin-top: var(--space-2); }

@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }

@media (max-width: 900px) {
  .content-grid { grid-template-columns: 1fr; }
  .detail-panel { position: static; }
}
@media (max-width: 640px) {
  .form-row { grid-template-columns: 1fr; }
}
</style>
