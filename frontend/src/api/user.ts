// ============================================================
// User API Service
// Sesuai dengan endpoint di src/handlers/user_handler.rs
// ============================================================

const BASE_URL = import.meta.env.VITE_API_URL ?? ''

import { authHeaders, authStorage, type ApiError } from './auth'

// ─── Response Types (sesuai dto/user.rs) ─────────────────

export interface UserProfileResponse {
  id: string
  name: string
  email: string
  phone: string | null
  photo: string | null
  address: string | null
  lat: number | null
  lng: number | null
  payment_token: string | null
  is_verified: boolean
  roles: string[]
}

export interface UpdateUserRequest {
  name?: string
  phone?: string
  address?: string
  lat?: number
  lng?: number
  payment_token?: string
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

// ─── User API Functions ────────────────────────────────────

/**
 * GET /api/users/me
 * Ambil profil user yang sedang login
 */
export async function getMyProfile(): Promise<UserProfileResponse> {
  const res = await fetch(`${BASE_URL}/api/users/me`, {
    method: 'GET',
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<UserProfileResponse>(res)
}

/**
 * PUT /api/users/me
 * Update profil user (multipart/form-data)
 * Note: Jangan set Content-Type header — browser auto-set boundary untuk FormData
 */
export async function updateMyProfile(data: UpdateUserRequest): Promise<UserProfileResponse> {
  const formData = new FormData()
  if (data.name) formData.append('name', data.name)
  if (data.phone) formData.append('phone', data.phone)
  if (data.address) formData.append('address', data.address)
  if (data.lat !== undefined) formData.append('lat', String(data.lat))
  if (data.lng !== undefined) formData.append('lng', String(data.lng))
  if (data.payment_token) formData.append('payment_token', data.payment_token)

  // Hanya kirim Authorization header, JANGAN set Content-Type untuk FormData
  const headers: Record<string, string> = {}
  const token = authStorage.getToken()
  if (token) headers['Authorization'] = `Bearer ${token}`

  const res = await fetch(`${BASE_URL}/api/users/me`, {
    method: 'PUT',
    headers,
    credentials: 'include',
    body: formData,
  })
  return handleResponse<UserProfileResponse>(res)
}

/**
 * POST /api/users/me/photo
 * Upload foto profil (multipart/form-data)
 * Backend menerima field name "file" atau "photo"
 */
export async function uploadMyPhoto(file: File): Promise<UserProfileResponse> {
  const formData = new FormData()
  // Backend cek "file" dulu, lalu "photo" — kirim sebagai "file" untuk kompatibilitas
  formData.append('file', file)

  // Hanya kirim Authorization header, JANGAN set Content-Type untuk FormData
  const headers: Record<string, string> = {}
  const token = authStorage.getToken()
  if (token) headers['Authorization'] = `Bearer ${token}`

  const res = await fetch(`${BASE_URL}/api/users/me/photo`, {
    method: 'POST',
    headers,
    credentials: 'include',
    body: formData,
  })
  return handleResponse<UserProfileResponse>(res)
}
