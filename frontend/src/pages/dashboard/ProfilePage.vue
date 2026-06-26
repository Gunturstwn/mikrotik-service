<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { authStorage } from '@/api/auth'
import { getMyProfile, updateMyProfile, uploadMyPhoto, changePassword, type UserProfileResponse, type UpdateUserRequest, type ChangePasswordRequest } from '@/api/user'

const router = useRouter()
const user = ref<UserProfileResponse | null>(null)
const isLoading = ref(true)
const error = ref('')
const isEditing = ref(false)
const editForm = ref<UpdateUserRequest>({})
const isSaving = ref(false)
const saveMsg = ref('')
const isUploadingPhoto = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

// Change password
const showChangePw = ref(false)
const pwForm = ref<ChangePasswordRequest>({ current_password: '', new_password: '' })
const pwConfirm = ref('')
const isChangingPw = ref(false)
const pwError = ref('')
const pwSuccess = ref('')

const initials = computed(() => {
  if (!user.value) return '?'
  return user.value.name.split(' ').map(w => w[0]).slice(0, 2).join('').toUpperCase()
})

const fetchProfile = async () => {
  isLoading.value = true
  error.value = ''
  try {
    user.value = await getMyProfile()
  } catch (e: any) {
    if (e.status === 401) { authStorage.clear(); router.push('/login'); return }
    error.value = e.message ?? 'Gagal memuat profil.'
  } finally { isLoading.value = false }
}

const startEdit = () => {
  if (!user.value) return
  editForm.value = { name: user.value.name, phone: user.value.phone ?? '', address: user.value.address ?? '' }
  isEditing.value = true; saveMsg.value = ''
}
const cancelEdit = () => { isEditing.value = false; saveMsg.value = '' }

const saveProfile = async () => {
  isSaving.value = true; saveMsg.value = ''
  try {
    user.value = await updateMyProfile(editForm.value)
    isEditing.value = false
    saveMsg.value = 'Profil berhasil diperbarui.'
    setTimeout(() => { saveMsg.value = '' }, 3000)
  } catch (e: any) { saveMsg.value = e.message ?? 'Gagal menyimpan.' }
  finally { isSaving.value = false }
}

const triggerPhotoUpload = () => { fileInput.value?.click() }
const photoError = ref('')
const handlePhotoChange = async (event: Event) => {
  const file = (event.target as HTMLInputElement).files?.[0]
  if (!file) return

  // Validasi client-side
  if (!file.type.startsWith('image/')) {
    photoError.value = 'File harus berupa gambar (JPG, PNG, WebP, dll).'
    return
  }
  if (file.size > 5 * 1024 * 1024) {
    photoError.value = 'Ukuran file maksimal 5MB.'
    return
  }

  photoError.value = ''
  isUploadingPhoto.value = true
  try {
    const result = await uploadMyPhoto(file)
    user.value = result
    photoError.value = ''
  } catch (e: any) {
    photoError.value = e.message ?? 'Gagal upload foto. Coba lagi.'
  } finally {
    isUploadingPhoto.value = false
    ;(event.target as HTMLInputElement).value = ''
  }
}

const handleChangePw = async () => {
  pwError.value = ''; pwSuccess.value = ''
  if (pwForm.value.new_password.length < 6) {
    pwError.value = 'Password baru minimal 6 karakter.'
    return
  }
  if (pwForm.value.new_password !== pwConfirm.value) {
    pwError.value = 'Konfirmasi password tidak cocok.'
    return
  }
  isChangingPw.value = true
  try {
    await changePassword(pwForm.value)
    pwSuccess.value = 'Password berhasil diganti.'
    setTimeout(() => {
      pwForm.value = { current_password: '', new_password: '' }
      pwConfirm.value = ''
      pwSuccess.value = ''
      showChangePw.value = false
    }, 2000)
  } catch (e: any) {
    pwError.value = e.message ?? 'Gagal mengganti password.'
  } finally { isChangingPw.value = false }
}

const cancelChangePw = () => {
  showChangePw.value = false
  pwForm.value = { current_password: '', new_password: '' }
  pwConfirm.value = ''
  pwError.value = ''
  pwSuccess.value = ''
}

onMounted(fetchProfile)
</script>

<template>
  <div class="profile-page">
    <h1 class="page-title">Profil Saya</h1>

    <!-- Loading -->
    <div v-if="isLoading" class="loading">
      <svg class="spin" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
    </div>

    <!-- Error -->
    <div v-else-if="error && !user" class="error-box glass-card">
      <p>{{ error }}</p>
      <button class="btn btn-primary" @click="fetchProfile">Coba Lagi</button>
    </div>

    <!-- Profile -->
    <div v-else-if="user" class="profile-card glass-card">
      <div class="avatar-section">
        <div class="avatar-wrap" @click="triggerPhotoUpload">
          <img v-if="user.photo" :src="user.photo" :alt="user.name" class="avatar-img" @error="($event.target as HTMLImageElement).style.display='none'" />
          <div v-if="!user.photo" class="avatar-placeholder">{{ initials }}</div>
          <div class="avatar-overlay">
            <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M23 19a2 2 0 0 1-2 2H3a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h4l2-3h6l2 3h4a2 2 0 0 1 2 2z"/><circle cx="12" cy="13" r="4"/>
            </svg>
          </div>
          <div v-if="isUploadingPhoto" class="avatar-loading">
            <svg class="spin" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="2.5"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg>
          </div>
        </div>
        <input ref="fileInput" type="file" accept="image/*" class="hidden" @change="handlePhotoChange" />
        <!-- Photo upload error -->
        <div v-if="photoError" class="photo-error">{{ photoError }}</div>
        <h2 class="profile-name">{{ user.name }}</h2>
        <p class="profile-email">{{ user.email }}</p>
        <div class="badges">
          <span v-for="role in user.roles" :key="role" class="badge role">{{ role }}</span>
          <span v-if="user.is_verified" class="badge verified">✓ Terverifikasi</span>
        </div>
      </div>

      <!-- Success msg -->
      <div v-if="saveMsg" class="save-msg">{{ saveMsg }}</div>

      <!-- View mode -->
      <div v-if="!isEditing" class="details">
        <div class="detail-row"><span class="label">Telepon</span><span class="value">{{ user.phone || '—' }}</span></div>
        <div class="detail-row"><span class="label">Alamat</span><span class="value">{{ user.address || '—' }}</span></div>
        <div v-if="user.lat && user.lng" class="detail-row"><span class="label">Koordinat</span><span class="value">{{ user.lat }}, {{ user.lng }}</span></div>
        <button class="btn btn-secondary" @click="startEdit">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7"/>
            <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z"/>
          </svg>
          Edit Profil
        </button>
      </div>

      <!-- Edit mode -->
      <form v-else class="edit-form" @submit.prevent="saveProfile">
        <div class="form-group">
          <label class="form-label" for="edit-name">Nama Lengkap</label>
          <input id="edit-name" v-model="editForm.name" type="text" class="form-input" />
        </div>
        <div class="form-group">
          <label class="form-label" for="edit-phone">Telepon</label>
          <input id="edit-phone" v-model="editForm.phone" type="tel" class="form-input" placeholder="+62 8xx" />
        </div>
        <div class="form-group">
          <label class="form-label" for="edit-address">Alamat</label>
          <textarea id="edit-address" v-model="editForm.address" class="form-textarea" rows="3"></textarea>
        </div>
        <div class="edit-actions">
          <button type="submit" class="btn btn-primary" :disabled="isSaving">{{ isSaving ? 'Menyimpan...' : 'Simpan' }}</button>
          <button type="button" class="btn btn-secondary" @click="cancelEdit">Batal</button>
        </div>
      </form>

      <!-- Change Password Section -->
      <div class="pw-section">
        <button v-if="!showChangePw" class="btn btn-secondary btn-pw" @click="showChangePw = true">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
          </svg>
          Ganti Password
        </button>

        <div v-if="showChangePw" class="pw-form">
          <h3 class="pw-title">Ganti Password</h3>
          <form @submit.prevent="handleChangePw">
            <div class="form-group">
              <label class="form-label" for="pw-current">Password Saat Ini</label>
              <input id="pw-current" v-model="pwForm.current_password" type="password" class="form-input" required />
            </div>
            <div class="form-group">
              <label class="form-label" for="pw-new">Password Baru</label>
              <input id="pw-new" v-model="pwForm.new_password" type="password" class="form-input" required minlength="6" />
            </div>
            <div class="form-group">
              <label class="form-label" for="pw-confirm">Konfirmasi Password Baru</label>
              <input id="pw-confirm" v-model="pwConfirm" type="password" class="form-input" required minlength="6" />
            </div>
            <div v-if="pwError" class="alert-error">{{ pwError }}</div>
            <div v-if="pwSuccess" class="alert-success">{{ pwSuccess }}</div>
            <div class="pw-actions">
              <button type="submit" class="btn btn-primary" :disabled="isChangingPw">{{ isChangingPw ? 'Menyimpan...' : 'Simpan Password' }}</button>
              <button type="button" class="btn btn-secondary" @click="cancelChangePw">Batal</button>
            </div>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.profile-page { max-width: 600px; }
.page-title { font-size: var(--font-size-2xl); font-weight: 800; margin-bottom: var(--space-6); }
.loading { display: flex; justify-content: center; padding: var(--space-16); }
.error-box { padding: var(--space-8); text-align: center; }
.profile-card { padding: var(--space-8); }
.avatar-section { display: flex; flex-direction: column; align-items: center; gap: var(--space-3); margin-bottom: var(--space-6); padding-bottom: var(--space-6); border-bottom: 1px solid var(--color-border); }
.avatar-wrap { position: relative; width: 80px; height: 80px; border-radius: 50%; cursor: pointer; overflow: hidden; }
.avatar-img { width: 100%; height: 100%; object-fit: cover; }
.avatar-placeholder { width: 100%; height: 100%; display: flex; align-items: center; justify-content: center; background: var(--gradient-primary); color: white; font-size: var(--font-size-2xl); font-weight: 800; }
.avatar-overlay { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; background: rgba(0,0,0,0.5); opacity: 0; transition: opacity var(--transition-base); color: white; }
.avatar-wrap:hover .avatar-overlay { opacity: 1; }
.avatar-loading { position: absolute; inset: 0; display: flex; align-items: center; justify-content: center; background: rgba(0,0,0,0.6); }
.hidden { display: none; }
.photo-error { font-size: var(--font-size-xs); color: #f87171; background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.2); padding: var(--space-2) var(--space-3); border-radius: var(--radius-md); text-align: center; max-width: 280px; }
.profile-name { font-size: var(--font-size-xl); font-weight: 800; }
.profile-email { font-size: var(--font-size-sm); color: var(--color-text-secondary); }
.badges { display: flex; gap: var(--space-2); flex-wrap: wrap; }
.badge { font-size: var(--font-size-xs); font-weight: 600; padding: 2px var(--space-3); border-radius: var(--radius-full); }
.badge.role { background: rgba(59,130,246,0.12); border: 1px solid rgba(59,130,246,0.25); color: var(--color-blue-light); }
.badge.verified { background: rgba(16,185,129,0.12); border: 1px solid rgba(16,185,129,0.25); color: var(--color-emerald); }
.save-msg { padding: var(--space-3) var(--space-4); background: rgba(16,185,129,0.1); border: 1px solid rgba(16,185,129,0.25); border-radius: var(--radius-md); color: var(--color-emerald); font-size: var(--font-size-sm); margin-bottom: var(--space-4); }
.details { display: flex; flex-direction: column; gap: var(--space-3); }
.detail-row { display: flex; justify-content: space-between; padding: var(--space-3) 0; border-bottom: 1px solid rgba(255,255,255,0.04); }
.label { font-size: var(--font-size-sm); font-weight: 600; color: var(--color-text-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.value { font-size: var(--font-size-sm); color: var(--color-text-primary); }
.edit-form { display: flex; flex-direction: column; gap: var(--space-4); }
.edit-form .form-textarea { min-height: 80px; }
.edit-actions { display: flex; gap: var(--space-3); }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }

/* Password Section */
.pw-section { margin-top: var(--space-8); padding-top: var(--space-6); border-top: 1px solid var(--color-border); }
.btn-pw { width: 100%; justify-content: center; }
.pw-form { margin-top: var(--space-4); padding: var(--space-5); background: rgba(255,255,255,0.03); border: 1px solid var(--color-border); border-radius: var(--radius-lg); }
.pw-title { font-size: var(--font-size-base); font-weight: 700; margin-bottom: var(--space-4); }
.alert-error { padding: var(--space-3) var(--space-4); background: rgba(239,68,68,0.1); border: 1px solid rgba(239,68,68,0.25); border-radius: var(--radius-md); color: #fca5a5; font-size: var(--font-size-sm); margin-bottom: var(--space-4); }
.alert-success { padding: var(--space-3) var(--space-4); background: rgba(16,185,129,0.1); border: 1px solid rgba(16,185,129,0.25); border-radius: var(--radius-md); color: var(--color-emerald); font-size: var(--font-size-sm); margin-bottom: var(--space-4); }
.pw-actions { display: flex; gap: var(--space-3); }
</style>
