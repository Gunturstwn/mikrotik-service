<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { authHeaders } from '@/api/auth'
import { listTelegramBots, type TelegramBotResponse } from '@/api/telegram'
import {
  listMikrotikClients,
  createMikrotikClient,
  deleteMikrotikClient,
  getMikrotikResource,
  testMikrotikConnection,
  triggerBackup,
  listBackupFiles,
  downloadBackupFileUrl,
  backupAndSend,
  type MikrotikClientResponse,
  type MikrotikClientRequest,
  type MikrotikResourceResponse,
  type BackupFileResponse,
  type BackupAndSendResponse,
} from '@/api/mikrotik'

const devices = ref<MikrotikClientResponse[]>([])
const isLoading = ref(true)
const error = ref('')

// Add device modal
const showAddModal = ref(false)
const addForm = ref<MikrotikClientRequest>({
  name_device: '', host: '', username: '', password: '',
  port_winbox: '', port_api: '', port_ftp: '', port_ssh: '',
  location: '', telegram_bot_id: '',
})
const isAdding = ref(false)
const addError = ref('')

// Device detail / resource
const selectedDevice = ref<MikrotikClientResponse | null>(null)
const deviceResource = ref<MikrotikResourceResponse | null>(null)
const isLoadingResource = ref(false)
const connectionStatus = ref<Record<string, boolean | null>>({})

// Backup state
const backupFiles = ref<BackupFileResponse[]>([])
const isLoadingBackupFiles = ref(false)
const isCreatingBackup = ref(false)
const backupName = ref('')
const backupPassword = ref('')
const backupError = ref('')

// Backup & Send state
const backupFormat = ref<'backup' | 'rsc'>('backup')
const sendTelegramBotId = ref('')
const deleteAfterSend = ref(true)
const isBackupAndSending = ref(false)
const sendResult = ref<BackupAndSendResponse | null>(null)

const telegramBots = ref<TelegramBotResponse[]>([])
const isLoadingBots = ref(false)

const fetchTelegramBots = async () => {
  isLoadingBots.value = true
  try {
    telegramBots.value = await listTelegramBots()
  } catch {
    telegramBots.value = []
  } finally {
    isLoadingBots.value = false
  }
}

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
    addForm.value = { name_device: '', host: '', username: '', password: '', port_winbox: '', port_api: '', port_ftp: '', port_ssh: '', location: '', telegram_bot_id: '' }
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
  // Load backup files
  fetchBackupFiles(device.id)
}

const checkConnection = async (id: string) => {
  connectionStatus.value[id] = null
  try {
    const ok = await testMikrotikConnection(id)
    connectionStatus.value[id] = ok
  } catch { connectionStatus.value[id] = false }
}

// ── Backup Functions ──────────────────────────────────────

const fetchBackupFiles = async (deviceId: string) => {
  isLoadingBackupFiles.value = true
  try {
    backupFiles.value = await listBackupFiles(deviceId)
  } catch {
    backupFiles.value = []
  } finally {
    isLoadingBackupFiles.value = false
  }
}

const handleCreateBackup = async () => {
  if (!selectedDevice.value) return
  isCreatingBackup.value = true; backupError.value = ''
  try {
    const result = await triggerBackup(selectedDevice.value.id, {
      name: backupName.value || undefined,
      password: backupPassword.value || undefined,
    })
    await fetchBackupFiles(selectedDevice.value.id)
    backupName.value = ''; backupPassword.value = ''
    // Show success briefly
    backupError.value = `✅ Backup berhasil: ${result.filename}`
    setTimeout(() => { backupError.value = '' }, 3000)
  } catch (e: any) {
    backupError.value = e.message ?? 'Gagal membuat backup.'
  } finally {
    isCreatingBackup.value = false
  }
}

const handleDownloadBackup = (filename: string) => {
  if (!selectedDevice.value) return
  // Create a temp auth header and trigger download via iframe or link
  // For simplicity, use window.open with the auth API URL
  const url = downloadBackupFileUrl(selectedDevice.value.id, filename)
  // Fetch with auth headers and trigger download programmatically
  fetch(url, { headers: authHeaders(), credentials: 'include' })
    .then(res => {
      if (!res.ok) throw new Error('Download failed')
      return res.blob()
    })
    .then(blob => {
      const a = document.createElement('a')
      a.href = URL.createObjectURL(blob)
      a.download = filename
      document.body.appendChild(a)
      a.click()
      a.remove()
      URL.revokeObjectURL(a.href)
    })
    .catch(e => alert(`Download gagal: ${e.message}`))
}

// ── Backup & Send Functions ──────────────────────────────

const getDefaultBotId = (): string => {
  if (!selectedDevice.value?.telegram_bot_id) return ''
  return selectedDevice.value.telegram_bot_id
}

const handleBackupAndSend = async () => {
  if (!selectedDevice.value) return
  const botId = sendTelegramBotId.value || getDefaultBotId()
  if (!botId) {
    sendResult.value = {
      filename: '', format: backupFormat.value,
      telegram_bot_id: '', telegram_success: false,
      telegram_message: 'Pilih bot Telegram terlebih dahulu',
      deleted_from_device: false,
    }
    return
  }
  isBackupAndSending.value = true; sendResult.value = null
  try {
    const result = await backupAndSend(selectedDevice.value.id, {
      name: backupName.value || undefined,
      password: backupPassword.value || undefined,
      format: backupFormat.value,
      telegram_bot_id: botId,
      delete_after_send: deleteAfterSend.value,
    })
    sendResult.value = result
    if (result.telegram_success) {
      backupName.value = ''; backupPassword.value = ''
      fetchBackupFiles(selectedDevice.value.id)
    }
  } catch (e: any) {
    sendResult.value = {
      filename: '', format: backupFormat.value,
      telegram_bot_id: botId, telegram_success: false,
      telegram_message: e.message ?? 'Gagal',
      deleted_from_device: false,
    }
  } finally {
    isBackupAndSending.value = false
  }
}

const formatBackupSize = (bytes: number) => {
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(1)} MB`
  if (bytes >= 1024) return `${(bytes / 1024).toFixed(0)} KB`
  return `${bytes} B`
}

const formatBytes = (bytes: number) => {
  if (bytes >= 1073741824) return `${(bytes / 1073741824).toFixed(1)} GB`
  if (bytes >= 1048576) return `${(bytes / 1048576).toFixed(0)} MB`
  return `${(bytes / 1024).toFixed(0)} KB`
}

onMounted(() => {
  fetchDevices()
  fetchTelegramBots()
})
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

        <!-- ── Backup Section ────────────────────────────────── -->
        <div class="backup-section">
          <h4 class="section-title">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"/>
            </svg>
            Backup
          </h4>

          <!-- Format Selector -->
          <div class="bf-format-selector">
            <label class="bf-format-option" :class="{ active: backupFormat === 'backup' }">
              <input type="radio" value="backup" v-model="backupFormat" />
              <span>.backup</span>
              <small>Binary</small>
            </label>
            <label class="bf-format-option" :class="{ active: backupFormat === 'rsc' }">
              <input type="radio" value="rsc" v-model="backupFormat" />
              <span>.rsc</span>
              <small>Export</small>
            </label>
          </div>

          <div class="backup-form">
            <div class="bf-row">
              <input v-model="backupName" class="form-input" placeholder="Nama backup (opsional)" />
              <input v-if="backupFormat === 'backup'" v-model="backupPassword" type="password" class="form-input" placeholder="Password (opsional)" />
            </div>

            <!-- Bot Selector -->
            <div class="bf-telegram-row">
              <select v-model="sendTelegramBotId" class="form-input">
                <option value="">— Pilih bot Telegram —</option>
                <option v-for="bot in telegramBots" :key="bot.id" :value="bot.id">
                  {{ bot.name }} ({{ bot.chat_id }})
                </option>
              </select>
              <label class="bf-checkbox">
                <input type="checkbox" v-model="deleteAfterSend" />
                Hapus dari device
              </label>
            </div>

            <div class="bf-actions">
              <button class="btn btn-primary" :disabled="isCreatingBackup" @click="handleCreateBackup" title="Simpan di filesystem device">
                <svg v-if="isCreatingBackup" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <polyline points="23 4 23 10 17 10"/><path d="M20.49 15a9 9 0 1 1-2.12-9.36L23 10"/>
                </svg>
                {{ isCreatingBackup ? 'Menyimpan...' : 'Simpan Backup' }}
              </button>
              <button class="btn btn-secondary" :disabled="isBackupAndSending" @click="handleBackupAndSend">
                <svg v-if="isBackupAndSending" class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
                <svg v-else width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <path d="M22 2L11 13"/><path d="M22 2l-7 20-4-9-9-4 20-7z"/>
                </svg>
                {{ isBackupAndSending ? 'Mengirim...' : 'Backup & Kirim' }}
              </button>
            </div>
          </div>

          <!-- Backup & Send Result -->
          <div v-if="sendResult" class="bf-send-result" :class="{ success: sendResult.telegram_success }">
            <div class="bf-send-icon">
              {{ sendResult.telegram_success ? '✅' : '❌' }}
            </div>
            <div class="bf-send-info">
              <span class="bf-send-file">{{ sendResult.filename }}</span>
              <span class="bf-send-msg">{{ sendResult.telegram_message || (sendResult.telegram_success ? 'Berhasil' : 'Gagal') }}</span>
              <span v-if="sendResult.deleted_from_device" class="bf-send-deleted">🗑️ Dihapus dari device</span>
            </div>
          </div>

          <div v-if="backupError" class="backup-msg" :class="{ success: backupError.startsWith('✅') }">{{ backupError }}</div>

          <!-- Backup File List -->
          <div v-if="isLoadingBackupFiles" class="loading-sm">
            <svg class="spin" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
            <span>Memuat file backup...</span>
          </div>

          <div v-else-if="backupFiles.length === 0" class="bf-empty">
            <span>Belum ada file backup.</span>
          </div>

          <div v-else class="backup-file-list">
            <div v-for="file in backupFiles" :key="file.name" class="bf-item">
              <div class="bf-info">
                <span class="bf-name">{{ file.name }}</span>
                <span class="bf-meta">{{ formatBackupSize(file.size) }} · {{ file.creation_time }}</span>
              </div>
              <button class="btn-download" @click="handleDownloadBackup(file.name)" title="Download">
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"/><polyline points="7 10 12 15 17 10"/><line x1="12" y1="15" x2="12" y2="3"/>
                </svg>
              </button>
            </div>
          </div>
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
            <div class="form-group">
              <label class="form-label">Telegram Bot Default</label>
              <select v-model="addForm.telegram_bot_id" class="form-input">
                <option value="">— Tidak ada —</option>
                <option v-for="bot in telegramBots" :key="bot.id" :value="bot.id">
                  {{ bot.name }} ({{ bot.chat_id }})
                </option>
              </select>
            </div>

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

/* Backup Section */
.backup-section { margin-top: var(--space-5); padding-top: var(--space-5); border-top: 1px solid var(--color-border); }
.section-title { font-size: var(--font-size-sm); font-weight: 700; margin-bottom: var(--space-4); display: flex; align-items: center; gap: var(--space-2); color: var(--color-text-primary); }
.backup-form { display: flex; flex-direction: column; gap: var(--space-3); margin-bottom: var(--space-4); }

/* Format Selector */
.bf-format-selector { display: flex; gap: var(--space-2); margin-bottom: var(--space-3); }
.bf-format-option { display: flex; align-items: center; gap: var(--space-2); padding: var(--space-2) var(--space-3); border: 1px solid var(--color-border); border-radius: var(--radius-md); cursor: pointer; transition: all var(--transition-base); font-size: var(--font-size-xs); }
.bf-format-option input { display: none; }
.bf-format-option span { font-weight: 700; color: var(--color-text-secondary); }
.bf-format-option small { color: var(--color-text-muted); font-size: 10px; }
.bf-format-option.active { border-color: var(--color-cyan); background: rgba(6,182,212,0.08); }
.bf-format-option.active span { color: var(--color-cyan); }

.bf-row { display: grid; grid-template-columns: 1fr 1fr; gap: var(--space-3); }
.bf-telegram-row { display: flex; gap: var(--space-3); align-items: center; }
.bf-telegram-row select { flex: 1; }
.bf-checkbox { display: flex; align-items: center; gap: var(--space-2); font-size: var(--font-size-xs); color: var(--color-text-secondary); white-space: nowrap; cursor: pointer; }
.bf-checkbox input { accent-color: var(--color-cyan); }
.bf-actions { display: flex; gap: var(--space-3); flex-wrap: wrap; }
.bf-actions .btn { display: flex; align-items: center; gap: var(--space-2); }

/* Send Result */
.bf-send-result { display: flex; gap: var(--space-3); padding: var(--space-3) var(--space-4); background: rgba(239,68,68,0.08); border: 1px solid rgba(239,68,68,0.2); border-radius: var(--radius-md); margin-bottom: var(--space-3); }
.bf-send-result.success { background: rgba(16,185,129,0.08); border-color: rgba(16,185,129,0.2); }
.bf-send-icon { font-size: var(--font-size-lg); flex-shrink: 0; }
.bf-send-info { display: flex; flex-direction: column; gap: 2px; }
.bf-send-file { font-size: var(--font-size-xs); font-weight: 700; color: var(--color-text-primary); }
.bf-send-msg { font-size: var(--font-size-xs); color: var(--color-text-secondary); }
.bf-send-deleted { font-size: 10px; color: var(--color-emerald); }

.backup-msg { font-size: var(--font-size-xs); padding: var(--space-2) var(--space-3); border-radius: var(--radius-sm); background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.2); color: #fca5a5; margin-bottom: var(--space-3); }
.backup-msg.success { background: rgba(16,185,129,0.1); border-color: rgba(16,185,129,0.25); color: var(--color-emerald); }
.bf-empty { font-size: var(--font-size-xs); color: var(--color-text-muted); padding: var(--space-3) 0; }
.backup-file-list { display: flex; flex-direction: column; gap: var(--space-2); max-height: 240px; overflow-y: auto; }
.bf-item { display: flex; justify-content: space-between; align-items: center; padding: var(--space-2) var(--space-3); background: rgba(255,255,255,0.03); border: 1px solid var(--color-border); border-radius: var(--radius-md); gap: var(--space-2); }
.bf-info { display: flex; flex-direction: column; min-width: 0; flex: 1; }
.bf-name { font-size: var(--font-size-xs); font-weight: 600; color: var(--color-text-primary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.bf-meta { font-size: 10px; color: var(--color-text-muted); }
.btn-download { width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; border-radius: var(--radius-sm); background: rgba(255,255,255,0.05); border: 1px solid var(--color-border); color: var(--color-cyan); cursor: pointer; transition: all var(--transition-base); flex-shrink: 0; }
.btn-download:hover { background: rgba(6,182,212,0.1); border-color: rgba(6,182,212,0.3); color: var(--color-cyan); }

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
