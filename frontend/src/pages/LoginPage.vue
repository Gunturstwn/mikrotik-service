<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { loginApi, getLoginStatusApi, authStorage, type ApiError } from '@/api/auth'

const router = useRouter()
const route = useRoute()

// Login form
const email = ref('')
const password = ref('')
const captchaToken = ref('')
const showPassword = ref(false)
const isLoading = ref(false)
const errorMsg = ref('')
const captchaRequired = ref(false)
const blockedUntil = ref<number | null>(null)

// Check login status when email changes (debounced)
let statusTimer: ReturnType<typeof setTimeout>
const checkStatus = () => {
  clearTimeout(statusTimer)
  if (!email.value.includes('@')) return
  statusTimer = setTimeout(async () => {
    try {
      const s = await getLoginStatusApi(email.value)
      captchaRequired.value = s.captcha_required
      blockedUntil.value = s.blocked_until
    } catch { /* ignore */ }
  }, 600)
}

const formatBlockTime = (secs: number) => {
  if (secs >= 3600) return `${Math.ceil(secs / 3600)} jam`
  if (secs >= 60) return `${Math.ceil(secs / 60)} menit`
  return `${secs} detik`
}

const handleLogin = async () => {
  errorMsg.value = ''
  if (!email.value || !password.value) {
    errorMsg.value = 'Email dan password wajib diisi.'
    return
  }
  isLoading.value = true
  try {
    const res = await loginApi({
      email: email.value,
      password: password.value,
      captcha_token: captchaToken.value || undefined,
    })
    authStorage.saveSession(res.token, res.user_id)
    const redirect = route.query.redirect as string | undefined
    router.push(redirect ?? '/dashboard')
  } catch (e) {
    const err = e as ApiError
    if (err.status === 400 && err.message?.toLowerCase().includes('captcha')) {
      captchaRequired.value = true
      errorMsg.value = 'Verifikasi CAPTCHA diperlukan.'
    } else if (err.status === 403) {
      errorMsg.value = `Akun/IP diblokir sementara. ${blockedUntil.value ? `Coba lagi dalam ${formatBlockTime(blockedUntil.value)}.` : ''}`
    } else if (err.status === 429) {
      errorMsg.value = 'Terlalu banyak percobaan. Coba lagi nanti.'
    } else if (err.status === 401) {
      errorMsg.value = 'Email atau password salah.'
    } else {
      errorMsg.value = err.message ?? 'Terjadi kesalahan. Coba lagi.'
    }
  } finally {
    isLoading.value = false
  }
}

onMounted(() => {
  if (authStorage.isLoggedIn()) router.push('/dashboard')
})
</script>

<template>
  <div class="login-root">
    <!-- Background -->
    <div class="login-bg" aria-hidden="true">
      <div class="bg-orb orb-1"></div>
      <div class="bg-orb orb-2"></div>
      <div class="bg-grid"></div>
    </div>

    <!-- Back to home -->
    <router-link to="/" class="back-home">
      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M19 12H5M12 5l-7 7 7 7"/>
      </svg>
      Kembali ke Beranda
    </router-link>

    <div class="login-center">
      <!-- Card -->
      <div class="login-card">
        <!-- Logo -->
        <div class="login-logo">
          <div class="logo-icon">
            <svg width="36" height="36" viewBox="0 0 32 32" fill="none">
              <circle cx="16" cy="16" r="16" fill="url(#lg1)"/>
              <path d="M8 16 L16 8 L24 16 L16 24 Z" fill="white" opacity="0.9"/>
              <circle cx="16" cy="16" r="4" fill="url(#lg2)"/>
              <defs>
                <linearGradient id="lg1" x1="0" y1="0" x2="32" y2="32">
                  <stop offset="0%" stop-color="#06b6d4"/><stop offset="100%" stop-color="#3b82f6"/>
                </linearGradient>
                <linearGradient id="lg2" x1="0" y1="0" x2="32" y2="32">
                  <stop offset="0%" stop-color="#22d3ee"/><stop offset="100%" stop-color="#60a5fa"/>
                </linearGradient>
              </defs>
            </svg>
          </div>
          <span class="logo-text">My PC24</span>
        </div>

        <div class="card-header">
          <h1 class="card-title">Masuk ke Akun Anda</h1>
          <p class="card-sub">Portal pelanggan PC24 Telekomunikasi Indonesia</p>
        </div>

        <!-- Blocked Banner -->
        <div v-if="blockedUntil" class="alert alert-danger">
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/>
          </svg>
          Akun diblokir. Coba lagi dalam <strong>{{ formatBlockTime(blockedUntil) }}</strong>.
        </div>

        <form @submit.prevent="handleLogin" novalidate>
          <!-- Email -->
          <div class="form-group">
            <label class="form-label" for="login-email">Email</label>
            <div class="input-wrap">
              <svg class="input-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/>
              </svg>
              <input
                id="login-email"
                v-model="email"
                type="email"
                class="form-input with-icon"
                placeholder="email@perusahaan.com"
                autocomplete="email"
                @input="checkStatus"
                :disabled="isLoading"
                required
              />
            </div>
          </div>

          <!-- Password -->
          <div class="form-group">
            <label class="form-label" for="login-password">Password</label>
            <div class="input-wrap">
              <svg class="input-icon" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="3" y="11" width="18" height="11" rx="2" ry="2"/><path d="M7 11V7a5 5 0 0 1 10 0v4"/>
              </svg>
              <input
                id="login-password"
                v-model="password"
                :type="showPassword ? 'text' : 'password'"
                class="form-input with-icon with-suffix"
                placeholder="••••••••"
                autocomplete="current-password"
                :disabled="isLoading"
                required
              />
              <button type="button" class="eye-btn" @click="showPassword = !showPassword" tabindex="-1">
                <svg v-if="!showPassword" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>
                </svg>
                <svg v-else width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"/>
                  <line x1="1" y1="1" x2="23" y2="23"/>
                </svg>
              </button>
            </div>
          </div>

          <!-- CAPTCHA notice -->
          <div v-if="captchaRequired" class="alert alert-warning">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/>
            </svg>
            Verifikasi CAPTCHA diperlukan untuk melanjutkan.
          </div>

          <!-- Error -->
          <div v-if="errorMsg" class="alert alert-danger">
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <circle cx="12" cy="12" r="10"/><line x1="15" y1="9" x2="9" y2="15"/><line x1="9" y1="9" x2="15" y2="15"/>
            </svg>
            {{ errorMsg }}
          </div>

          <!-- Submit -->
          <button
            type="submit"
            id="login-submit-btn"
            class="btn btn-primary submit-btn"
            :disabled="isLoading || !!blockedUntil"
          >
            <svg v-if="isLoading" class="spin" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
            </svg>
            <span>{{ isLoading ? 'Memproses...' : 'Masuk' }}</span>
          </button>
        </form>
      </div>

      <p class="login-footer">
        © {{ new Date().getFullYear() }} PT PC24 Telekomunikasi Indonesia
      </p>
    </div>
  </div>
</template>

<style scoped>
/* Root */
.login-root {
  min-height: 100vh;
  background: var(--color-bg);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  overflow: hidden;
  padding: var(--space-6);
}

/* Bg */
.login-bg { position: absolute; inset: 0; pointer-events: none; }
.bg-orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
}
.orb-1 {
  width: 500px; height: 500px;
  background: radial-gradient(circle, rgba(6,182,212,0.15) 0%, transparent 70%);
  top: -150px; right: -100px;
  animation: float 9s ease-in-out infinite;
}
.orb-2 {
  width: 400px; height: 400px;
  background: radial-gradient(circle, rgba(59,130,246,0.12) 0%, transparent 70%);
  bottom: -100px; left: -100px;
  animation: float 11s ease-in-out infinite reverse;
}
.bg-grid {
  position: absolute; inset: 0;
  background-image:
    linear-gradient(rgba(6,182,212,0.03) 1px, transparent 1px),
    linear-gradient(90deg, rgba(6,182,212,0.03) 1px, transparent 1px);
  background-size: 50px 50px;
  mask-image: radial-gradient(ellipse 80% 80% at 50% 50%, black, transparent);
}

/* Back link */
.back-home {
  position: fixed;
  top: var(--space-6);
  left: var(--space-6);
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  padding: var(--space-2) var(--space-4);
  background: rgba(255,255,255,0.05);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-full);
  transition: all var(--transition-base);
  z-index: 10;
}
.back-home:hover { color: var(--color-cyan); border-color: rgba(6,182,212,0.4); }

/* Center */
.login-center {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 480px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
}

/* Card */
.login-card {
  width: 100%;
  background: rgba(13, 24, 41, 0.8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-2xl);
  backdrop-filter: blur(20px);
  padding: var(--space-10);
  box-shadow: 0 24px 64px rgba(0,0,0,0.5);
}

/* Logo */
.login-logo {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  margin-bottom: var(--space-8);
}
.logo-icon { animation: float 6s ease-in-out infinite; }
.logo-text {
  font-size: var(--font-size-2xl);
  font-weight: 800;
  letter-spacing: -0.02em;
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

/* Header */
.card-header { text-align: center; margin-bottom: var(--space-8); }
.card-title {
  font-size: var(--font-size-2xl);
  font-weight: 800;
  letter-spacing: -0.02em;
  margin-bottom: var(--space-2);
}
.card-sub { font-size: var(--font-size-sm); color: var(--color-text-secondary); }

/* Form */
.form-group { margin-bottom: var(--space-5); }

.input-wrap { position: relative; }
.input-icon {
  position: absolute;
  left: var(--space-4);
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-muted);
  pointer-events: none;
}
.form-input.with-icon { padding-left: calc(var(--space-4) * 2 + 16px); }
.form-input.with-suffix { padding-right: calc(var(--space-4) * 2 + 16px); }
.eye-btn {
  position: absolute;
  right: var(--space-4);
  top: 50%;
  transform: translateY(-50%);
  color: var(--color-text-muted);
  transition: color var(--transition-base);
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
}
.eye-btn:hover { color: var(--color-text-primary); }

/* Alerts */
.alert {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
  border-radius: var(--radius-md);
  font-size: var(--font-size-sm);
  line-height: 1.6;
  margin-bottom: var(--space-5);
}
.alert svg { flex-shrink: 0; margin-top: 1px; }
.alert-danger {
  background: rgba(239,68,68,0.1);
  border: 1px solid rgba(239,68,68,0.25);
  color: #fca5a5;
}
.alert-warning {
  background: rgba(245,158,11,0.1);
  border: 1px solid rgba(245,158,11,0.25);
  color: #fcd34d;
}

/* Submit */
.submit-btn {
  width: 100%;
  justify-content: center;
  gap: var(--space-2);
}
.submit-btn:disabled { opacity: 0.6; cursor: not-allowed; transform: none !important; }

/* Spinner */
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }

/* Footer */
.login-footer { font-size: var(--font-size-xs); color: var(--color-text-muted); text-align: center; }
</style>
