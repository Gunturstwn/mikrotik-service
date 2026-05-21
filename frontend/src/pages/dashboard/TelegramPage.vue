<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  listTelegramBots, createTelegramBot, updateTelegramBot,
  deleteTelegramBot, testTelegramBot,
  type TelegramBotResponse, type CreateTelegramBotRequest, type UpdateTelegramBotRequest,
} from '@/api/telegram'

const bots = ref<TelegramBotResponse[]>([])
const isLoading = ref(true)
const error = ref('')

// Add modal
const showAddModal = ref(false)
const addForm = ref<CreateTelegramBotRequest>({ name: '', token: '', chat_id: '', description: '' })
const isAdding = ref(false)
const addError = ref('')

// Edit modal
const showEditModal = ref(false)
const editId = ref('')
const editForm = ref<UpdateTelegramBotRequest>({})
const isEditing = ref(false)
const editError = ref('')

// Test result
const testResult = ref<Record<string, { success: boolean; message: string } | null>>({})

const fetchBots = async () => {
  isLoading.value = true; error.value = ''
  try { bots.value = await listTelegramBots() }
  catch (e: any) { error.value = e.message ?? 'Gagal memuat data.' }
  finally { isLoading.value = false }
}

const handleAdd = async () => {
  isAdding.value = true; addError.value = ''
  try {
    const created = await createTelegramBot(addForm.value)
    bots.value.unshift(created)
    showAddModal.value = false
    addForm.value = { name: '', token: '', chat_id: '', description: '' }
  } catch (e: any) { addError.value = e.message ?? 'Gagal menambah bot.' }
  finally { isAdding.value = false }
}

const openEdit = (bot: TelegramBotResponse) => {
  editId.value = bot.id
  editForm.value = { name: bot.name, chat_id: bot.chat_id, description: bot.description ?? '', is_active: bot.is_active }
  editError.value = ''
  showEditModal.value = true
}

const handleEdit = async () => {
  isEditing.value = true; editError.value = ''
  try {
    const updated = await updateTelegramBot(editId.value, editForm.value)
    const idx = bots.value.findIndex(b => b.id === editId.value)
    if (idx >= 0) bots.value[idx] = updated
    showEditModal.value = false
  } catch (e: any) { editError.value = e.message ?? 'Gagal update.' }
  finally { isEditing.value = false }
}

const handleDelete = async (id: string) => {
  if (!confirm('Hapus bot Telegram ini?')) return
  try { await deleteTelegramBot(id); bots.value = bots.value.filter(b => b.id !== id) }
  catch (e: any) { alert(e.message ?? 'Gagal menghapus.') }
}

const handleTest = async (id: string) => {
  testResult.value[id] = null
  try {
    const res = await testTelegramBot(id)
    testResult.value[id] = res
  } catch (e: any) { testResult.value[id] = { success: false, message: e.message ?? 'Error' } }
}

onMounted(fetchBots)
</script>

<template>
  <div class="telegram-page">
    <div class="page-header">
      <div>
        <h1 class="page-title">Telegram Bots</h1>
        <p class="page-sub">Kelola bot Telegram untuk notifikasi dan alert.</p>
      </div>
      <button class="btn btn-primary" @click="showAddModal = true">
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
          <line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/>
        </svg>
        Tambah Bot
      </button>
    </div>

    <!-- Loading -->
    <div v-if="isLoading" class="loading">
      <svg class="spin" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
    </div>

    <!-- Error -->
    <div v-else-if="error" class="error-box glass-card">
      <p>{{ error }}</p>
      <button class="btn btn-primary" @click="fetchBots">Coba Lagi</button>
    </div>

    <!-- Bot List -->
    <div v-else class="bot-list">
      <div v-if="bots.length === 0" class="empty-state glass-card">
        <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/>
        </svg>
        <p>Belum ada bot Telegram. Tambahkan bot pertama Anda.</p>
      </div>

      <div v-for="bot in bots" :key="bot.id" class="bot-card glass-card">
        <div class="bot-header">
          <div class="bot-icon" :class="{ active: bot.is_active }">
            <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/>
            </svg>
          </div>
          <div class="bot-info">
            <span class="bot-name">{{ bot.name }}</span>
            <span class="bot-token">Token: {{ bot.token_masked }}</span>
          </div>
          <span class="bot-status" :class="bot.is_active ? 'active' : 'inactive'">
            {{ bot.is_active ? 'Aktif' : 'Nonaktif' }}
          </span>
        </div>

        <div class="bot-details">
          <div class="bot-detail"><span class="dl">Chat ID</span><span class="dv">{{ bot.chat_id }}</span></div>
          <div v-if="bot.description" class="bot-detail"><span class="dl">Deskripsi</span><span class="dv">{{ bot.description }}</span></div>
        </div>

        <!-- Test result -->
        <div v-if="testResult[bot.id]" class="test-result" :class="testResult[bot.id]?.success ? 'success' : 'fail'">
          {{ testResult[bot.id]?.message }}
        </div>

        <div class="bot-actions">
          <button class="action-btn test" @click="handleTest(bot.id)" title="Test kirim pesan">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M22 2L11 13M22 2l-7 20-4-9-9-4 20-7z"/>
            </svg>
            Test
          </button>
          <button class="action-btn" @click="openEdit(bot)" title="Edit">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
              <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
            </svg>
          </button>
          <button class="action-btn danger" @click="handleDelete(bot.id)" title="Hapus">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
            </svg>
          </button>
        </div>
      </div>
    </div>

    <!-- Add Modal -->
    <Teleport to="body">
      <div v-if="showAddModal" class="modal-overlay" @click.self="showAddModal = false">
        <div class="modal glass-card">
          <h2 class="modal-title">Tambah Bot Telegram</h2>
          <form @submit.prevent="handleAdd">
            <div class="form-group"><label class="form-label">Nama Bot *</label><input v-model="addForm.name" class="form-input" required placeholder="Alert Bot Production" /></div>
            <div class="form-group"><label class="form-label">Token Bot * (dari @BotFather)</label><input v-model="addForm.token" class="form-input" required placeholder="123456789:ABCdefGHI..." /></div>
            <div class="form-group"><label class="form-label">Chat ID *</label><input v-model="addForm.chat_id" class="form-input" required placeholder="-1001234567890" /></div>
            <div class="form-group"><label class="form-label">Deskripsi</label><input v-model="addForm.description" class="form-input" placeholder="Bot untuk notifikasi alert" /></div>
            <div v-if="addError" class="alert-error">{{ addError }}</div>
            <div class="modal-actions">
              <button type="submit" class="btn btn-primary" :disabled="isAdding">{{ isAdding ? 'Menyimpan...' : 'Simpan' }}</button>
              <button type="button" class="btn btn-secondary" @click="showAddModal = false">Batal</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>

    <!-- Edit Modal -->
    <Teleport to="body">
      <div v-if="showEditModal" class="modal-overlay" @click.self="showEditModal = false">
        <div class="modal glass-card">
          <h2 class="modal-title">Edit Bot Telegram</h2>
          <form @submit.prevent="handleEdit">
            <div class="form-group"><label class="form-label">Nama Bot</label><input v-model="editForm.name" class="form-input" placeholder="Alert Bot" /></div>
            <div class="form-group"><label class="form-label">Token Baru (kosongkan jika tidak diubah)</label><input v-model="editForm.token" class="form-input" placeholder="Biarkan kosong jika tidak diubah" /></div>
            <div class="form-group"><label class="form-label">Chat ID</label><input v-model="editForm.chat_id" class="form-input" /></div>
            <div class="form-group"><label class="form-label">Deskripsi</label><input v-model="editForm.description" class="form-input" /></div>
            <div class="form-group toggle-group">
              <label class="form-label">Status</label>
              <label class="toggle">
                <input type="checkbox" v-model="editForm.is_active" />
                <span class="toggle-slider"></span>
                <span class="toggle-label">{{ editForm.is_active ? 'Aktif' : 'Nonaktif' }}</span>
              </label>
            </div>
            <div v-if="editError" class="alert-error">{{ editError }}</div>
            <div class="modal-actions">
              <button type="submit" class="btn btn-primary" :disabled="isEditing">{{ isEditing ? 'Menyimpan...' : 'Simpan' }}</button>
              <button type="button" class="btn btn-secondary" @click="showEditModal = false">Batal</button>
            </div>
          </form>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.telegram-page { width: 100%; max-width: 800px; }
.page-header { display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: var(--space-6); flex-wrap: wrap; gap: var(--space-4); }
.page-title { font-size: var(--font-size-2xl); font-weight: 800; margin-bottom: var(--space-1); }
.page-sub { font-size: var(--font-size-sm); color: var(--color-text-secondary); }
.loading { display: flex; justify-content: center; padding: var(--space-16); }
.error-box { padding: var(--space-8); text-align: center; }
.empty-state { padding: var(--space-10); display: flex; flex-direction: column; align-items: center; gap: var(--space-4); color: var(--color-text-muted); text-align: center; }

.bot-list { display: flex; flex-direction: column; gap: var(--space-4); }
.bot-card { padding: var(--space-5) var(--space-6); }
.bot-header { display: flex; align-items: center; gap: var(--space-3); margin-bottom: var(--space-3); }
.bot-icon { width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; background: rgba(100,100,100,0.1); border: 1px solid var(--color-border); border-radius: var(--radius-md); color: var(--color-text-muted); flex-shrink: 0; }
.bot-icon.active { background: rgba(6,182,212,0.1); border-color: rgba(6,182,212,0.25); color: var(--color-cyan); }
.bot-info { flex: 1; min-width: 0; }
.bot-name { display: block; font-size: var(--font-size-sm); font-weight: 700; color: var(--color-text-primary); }
.bot-token { font-size: var(--font-size-xs); color: var(--color-text-muted); font-family: monospace; }
.bot-status { font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: var(--radius-full); }
.bot-status.active { background: rgba(16,185,129,0.15); color: var(--color-emerald); border: 1px solid rgba(16,185,129,0.3); }
.bot-status.inactive { background: rgba(100,100,100,0.1); color: var(--color-text-muted); border: 1px solid var(--color-border); }

.bot-details { display: flex; flex-direction: column; gap: var(--space-1); margin-bottom: var(--space-3); padding-left: 52px; }
.bot-detail { display: flex; gap: var(--space-3); font-size: var(--font-size-xs); }
.dl { color: var(--color-text-muted); min-width: 70px; }
.dv { color: var(--color-text-secondary); font-family: monospace; }

.test-result { font-size: var(--font-size-xs); padding: var(--space-2) var(--space-3); border-radius: var(--radius-sm); margin-bottom: var(--space-3); margin-left: 52px; }
.test-result.success { background: rgba(16,185,129,0.1); border: 1px solid rgba(16,185,129,0.2); color: var(--color-emerald); }
.test-result.fail { background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.2); color: #f87171; }

.bot-actions { display: flex; gap: var(--space-2); padding-left: 52px; }
.action-btn { display: inline-flex; align-items: center; gap: var(--space-1); padding: var(--space-1) var(--space-3); border-radius: var(--radius-sm); background: rgba(255,255,255,0.05); border: 1px solid var(--color-border); color: var(--color-text-secondary); font-size: var(--font-size-xs); font-weight: 500; cursor: pointer; transition: all var(--transition-base); }
.action-btn:hover { background: rgba(6,182,212,0.1); border-color: rgba(6,182,212,0.3); color: var(--color-cyan); }
.action-btn.test:hover { background: rgba(59,130,246,0.1); border-color: rgba(59,130,246,0.3); color: var(--color-blue); }
.action-btn.danger:hover { background: rgba(239,68,68,0.1); border-color: rgba(239,68,68,0.3); color: #f87171; }

/* Modal */
.modal-overlay { position: fixed; inset: 0; z-index: 1000; background: rgba(0,0,0,0.7); backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; padding: var(--space-4); }
.modal { width: 100%; max-width: 480px; padding: var(--space-8); max-height: 90vh; overflow-y: auto; }
.modal-title { font-size: var(--font-size-xl); font-weight: 800; margin-bottom: var(--space-6); }
.form-group { margin-bottom: var(--space-4); }
.alert-error { padding: var(--space-3) var(--space-4); background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.25); border-radius: var(--radius-md); color: #fca5a5; font-size: var(--font-size-sm); margin-bottom: var(--space-4); }
.modal-actions { display: flex; gap: var(--space-3); margin-top: var(--space-2); }

/* Toggle */
.toggle-group { display: flex; align-items: center; justify-content: space-between; }
.toggle { display: flex; align-items: center; gap: var(--space-2); cursor: pointer; }
.toggle input { display: none; }
.toggle-slider { width: 36px; height: 20px; background: rgba(100,100,100,0.3); border-radius: 10px; position: relative; transition: background var(--transition-base); }
.toggle-slider::after { content: ''; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px; background: white; border-radius: 50%; transition: transform var(--transition-base); }
.toggle input:checked + .toggle-slider { background: var(--color-cyan); }
.toggle input:checked + .toggle-slider::after { transform: translateX(16px); }
.toggle-label { font-size: var(--font-size-sm); color: var(--color-text-secondary); }

@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }
</style>
