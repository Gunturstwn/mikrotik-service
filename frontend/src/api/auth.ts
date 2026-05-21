// ============================================================
// Auth API Service
// Sesuai dengan endpoint di src/handlers/auth_handler.rs
// Base URL dikonfigurasi via VITE_API_URL env variable
//
// Strategy: Token disimpan di sessionStorage (tab-scoped, tidak
// accessible via JS dari domain lain). Dikirim via Authorization
// header. Ketika backend sudah support httpOnly cookie, frontend
// akan otomatis bekerja karena credentials: 'include' sudah aktif.
// ============================================================

const BASE_URL = import.meta.env.VITE_API_URL ?? ''

// ─── Request Types (sesuai dto/auth.rs) ──────────────────

/** POST /api/auth/login */
export interface LoginRequest {
  email: string
  password: string
  /** Cloudflare Turnstile token. Required jika captcha_required = true */
  captcha_token?: string
}

// ─── Response Types (sesuai dto/auth.rs) ─────────────────

/** Response dari POST /api/auth/login */
export interface AuthResponse {
  token: string
  user_id: string
}

/**
 * Response dari GET /api/auth/login-status
 */
export interface LoginStatusResponse {
  captcha_required: boolean
  blocked_until: number | null
}

/** Error response standar dari backend */
export interface ApiError {
  message: string
  status: number
}

// ─── Helper ───────────────────────────────────────────────

async function handleResponse<T>(res: Response): Promise<T> {
  if (res.ok) {
    if (res.status === 204) return undefined as unknown as T
    return res.json() as Promise<T>
  }

  let message = `HTTP ${res.status}`
  try {
    const body = await res.json()
    message = body.message ?? body.error ?? JSON.stringify(body)
  } catch {
    message = res.statusText || message
  }

  const err: ApiError = { message, status: res.status }
  throw err
}

/**
 * Buat headers dengan Authorization token jika tersedia.
 * Ketika backend sudah set httpOnly cookie, header ini menjadi fallback.
 */
export function authHeaders(extra?: Record<string, string>): Record<string, string> {
  const headers: Record<string, string> = { ...extra }
  const token = authStorage.getToken()
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }
  return headers
}

// ─── Auth API Functions ────────────────────────────────────

/**
 * POST /api/auth/login
 * Autentikasi user, return JWT token + user_id
 */
export async function loginApi(payload: LoginRequest): Promise<AuthResponse> {
  const res = await fetch(`${BASE_URL}/api/auth/login`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    credentials: 'include',
    body: JSON.stringify(payload),
  })
  return handleResponse<AuthResponse>(res)
}

/**
 * GET /api/auth/login-status?email=xxx
 */
export async function getLoginStatusApi(email: string): Promise<LoginStatusResponse> {
  const params = new URLSearchParams({ email })
  const res = await fetch(`${BASE_URL}/api/auth/login-status?${params}`, {
    method: 'GET',
    credentials: 'include',
  })
  return handleResponse<LoginStatusResponse>(res)
}

/**
 * POST /api/auth/logout
 * Logout user, clear session
 */
export async function logoutApi(): Promise<void> {
  try {
    await fetch(`${BASE_URL}/api/auth/logout`, {
      method: 'POST',
      headers: authHeaders(),
      credentials: 'include',
    })
  } catch {
    // Ignore network errors on logout
  }
}

// ─── Session Storage (tab-scoped, lebih aman dari localStorage) ──

export const authStorage = {
  saveSession(token: string, userId: string) {
    sessionStorage.setItem('pc24_token', token)
    sessionStorage.setItem('pc24_user_id', userId)
  },
  getToken(): string | null {
    return sessionStorage.getItem('pc24_token')
  },
  getUserId(): string | null {
    return sessionStorage.getItem('pc24_user_id')
  },
  clear() {
    sessionStorage.removeItem('pc24_token')
    sessionStorage.removeItem('pc24_user_id')
  },
  isLoggedIn(): boolean {
    return !!sessionStorage.getItem('pc24_token')
  },
}
