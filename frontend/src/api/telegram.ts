// ============================================================
// Telegram Bot API Service
// Sesuai dengan endpoint di src/handlers/telegram_handler.rs
// ============================================================

const BASE_URL = import.meta.env.VITE_API_URL ?? ''

import { authHeaders, type ApiError } from './auth'

// ─── Types ───────────────────────────────────────────────

export interface CreateTelegramBotRequest {
  name: string
  token: string
  chat_id: string
  description?: string
  is_active?: boolean
}

export interface UpdateTelegramBotRequest {
  name?: string
  token?: string
  chat_id?: string
  description?: string
  is_active?: boolean
}

export interface TelegramBotResponse {
  id: string
  name: string
  token_masked: string
  chat_id: string
  is_active: boolean
  description: string | null
  created_at: string
  updated_at: string
  created_by: string
  updated_by: string | null
}

export interface TelegramTestResponse {
  success: boolean
  message: string
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
  } catch { message = res.statusText || message }
  const err: ApiError = { message, status: res.status }
  throw err
}

// ─── API Functions ────────────────────────────────────────

export async function listTelegramBots(): Promise<TelegramBotResponse[]> {
  const res = await fetch(`${BASE_URL}/api/telegram`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<TelegramBotResponse[]>(res)
}

export async function getTelegramBot(id: string): Promise<TelegramBotResponse> {
  const res = await fetch(`${BASE_URL}/api/telegram/${id}`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<TelegramBotResponse>(res)
}

export async function createTelegramBot(data: CreateTelegramBotRequest): Promise<TelegramBotResponse> {
  const res = await fetch(`${BASE_URL}/api/telegram`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    credentials: 'include',
    body: JSON.stringify(data),
  })
  return handleResponse<TelegramBotResponse>(res)
}

export async function updateTelegramBot(id: string, data: UpdateTelegramBotRequest): Promise<TelegramBotResponse> {
  const res = await fetch(`${BASE_URL}/api/telegram/${id}`, {
    method: 'PUT',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    credentials: 'include',
    body: JSON.stringify(data),
  })
  return handleResponse<TelegramBotResponse>(res)
}

export async function deleteTelegramBot(id: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/api/telegram/${id}`, {
    method: 'DELETE',
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<void>(res)
}

export async function testTelegramBot(id: string): Promise<TelegramTestResponse> {
  const res = await fetch(`${BASE_URL}/api/telegram/${id}/test`, {
    method: 'POST',
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<TelegramTestResponse>(res)
}
