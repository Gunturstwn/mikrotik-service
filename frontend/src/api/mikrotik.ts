// ============================================================
// MikroTik API Service
// Sesuai dengan endpoint di src/handlers/mikrotik_handler.rs
// ============================================================

const BASE_URL = import.meta.env.VITE_API_URL ?? ''

import { authHeaders, type ApiError } from './auth'

// ─── Types (sesuai dto/mikrotik.rs) ──────────────────────

export interface MikrotikClientRequest {
  name_device: string
  host: string
  username: string
  password: string
  port_winbox?: string
  port_api?: string
  port_ftp?: string
  port_ssh?: string
  location?: string
  latitude?: number
  longitude?: number
  timezone?: string
  telegram_bot_id?: string
}

export interface MikrotikClientResponse {
  id: string
  name_device: string
  host: string
  username: string
  port_ssh: string | null
  port_winbox: string | null
  port_api: string | null
  port_ftp: string | null
  location: string | null
  latitude: number | null
  longitude: number | null
  timezone: string | null
  telegram_bot_id: string | null
  created_at: string
  updated_at: string
  created_by: string
  updated_by: string | null
}

export interface MikrotikResourceResponse {
  uptime: string
  cpu_load: number
  free_memory: number
  total_memory: number
  free_hdd_space: number
  total_hdd_space: number
}

export interface MikrotikInterfaceResponse {
  name: string
  default_name: string | null
  type_name: string | null
  mtu: number | null
  actual_mtu: number | null
  mac_address: string | null
  last_link_up_time: string | null
  link_downs: number | null
  rx_byte: number | null
  tx_byte: number | null
  rx_packet: number | null
  tx_packet: number | null
  rx_error: number | null
  tx_error: number | null
  rx_drop: number | null
  tx_drop: number | null
  running: boolean
  disabled: boolean
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

// ─── API Functions ────────────────────────────────────────

/** GET /api/mikrotik_client — List all devices */
export async function listMikrotikClients(): Promise<MikrotikClientResponse[]> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<MikrotikClientResponse[]>(res)
}

/** GET /api/mikrotik_client/:id — Get device detail */
export async function getMikrotikClient(id: string): Promise<MikrotikClientResponse> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<MikrotikClientResponse>(res)
}

/** POST /api/mikrotik_client — Create new device */
export async function createMikrotikClient(data: MikrotikClientRequest): Promise<MikrotikClientResponse> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    credentials: 'include',
    body: JSON.stringify(data),
  })
  return handleResponse<MikrotikClientResponse>(res)
}

/** PUT /api/mikrotik_client/:id — Update device */
export async function updateMikrotikClient(id: string, data: MikrotikClientRequest): Promise<MikrotikClientResponse> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}`, {
    method: 'PUT',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    credentials: 'include',
    body: JSON.stringify(data),
  })
  return handleResponse<MikrotikClientResponse>(res)
}

/** DELETE /api/mikrotik_client/:id — Delete device */
export async function deleteMikrotikClient(id: string): Promise<void> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}`, {
    method: 'DELETE',
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<void>(res)
}

/** GET /api/mikrotik_client/:id/system/resource/print — Get system resources */
export async function getMikrotikResource(id: string): Promise<MikrotikResourceResponse> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/system/resource/print`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<MikrotikResourceResponse>(res)
}

/** GET /api/mikrotik_client/:id/interfaces/print — Get interfaces */
export async function getMikrotikInterfaces(id: string): Promise<MikrotikInterfaceResponse[]> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/interfaces/print`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<MikrotikInterfaceResponse[]>(res)
}

/** GET /api/mikrotik_client/:id/test-connection — Test connectivity */
export async function testMikrotikConnection(id: string): Promise<boolean> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/test-connection`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return res.status === 200
}

// ─── Backup Types ─────────────────────────────────────────

export interface BackupCreateRequest {
  name?: string
  password?: string
}

export interface BackupFileResponse {
  name: string
  size: number
  creation_time: string
}

export interface BackupCreateResponse {
  filename: string
}

export type BackupFormat = 'backup' | 'rsc'

export interface BackupAndSendRequest {
  name?: string
  password?: string
  format?: BackupFormat
  telegram_bot_id: string
  delete_after_send?: boolean
}

export interface BackupAndSendResponse {
  filename: string
  format: string
  telegram_bot_id: string
  telegram_success: boolean
  telegram_message: string | null
  deleted_from_device: boolean
}

// ─── Backup API Functions ────────────────────────────────

/** POST /api/mikrotik_client/:id/backup — Trigger binary backup */
export async function triggerBackup(id: string, data?: BackupCreateRequest): Promise<BackupCreateResponse> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/backup`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    credentials: 'include',
    body: JSON.stringify(data ?? {}),
  })
  return handleResponse<BackupCreateResponse>(res)
}

/** POST /api/mikrotik_client/:id/backup/send — Backup & send to Telegram */
export async function backupAndSend(id: string, data: BackupAndSendRequest): Promise<BackupAndSendResponse> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/backup/send`, {
    method: 'POST',
    headers: authHeaders({ 'Content-Type': 'application/json' }),
    credentials: 'include',
    body: JSON.stringify(data),
  })
  return handleResponse<BackupAndSendResponse>(res)
}

/** GET /api/mikrotik_client/:id/backup/files — List backup files */
export async function listBackupFiles(id: string): Promise<BackupFileResponse[]> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/backup/files`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<BackupFileResponse[]>(res)
}

/** GET /api/mikrotik_client/:id/backup/download?filename=... — Download backup file */
export async function downloadBackupFileUrl(id: string, filename: string): Promise<string> {
  // Return the URL directly for browser download
  return `${BASE_URL}/api/mikrotik_client/${id}/backup/download?filename=${encodeURIComponent(filename)}`
}

/** GET /api/mikrotik_client/backup-logs — List backup activity logs */
export interface BackupLogResponse {
  id: string
  device_name: string
  device_host: string
  filename: string
  format: string | null
  telegram_bot_name: string | null
  telegram_success: boolean
  deleted_from_device: boolean
  user_name: string | null
  created_at: string
}

export interface BackupLogListResponse {
  items: BackupLogResponse[]
  total: number
  page: number
  page_size: number
  total_pages: number
}

export async function listBackupLogs(page = 1, pageSize = 20, deviceId?: string): Promise<BackupLogListResponse> {
  let url = `${BASE_URL}/api/mikrotik_client/backup-logs?page=${page}&page_size=${pageSize}`
  if (deviceId) { url += `&device_id=${encodeURIComponent(deviceId)}` }
  const res = await fetch(url, {
    headers: authHeaders(),
    credentials: 'include',
  })
  return handleResponse<BackupLogListResponse>(res)
}

/** Download backup file as Blob for programmatic use */
export async function downloadBackupFileBlob(id: string, filename: string): Promise<Blob> {
  const res = await fetch(`${BASE_URL}/api/mikrotik_client/${id}/backup/download?filename=${encodeURIComponent(filename)}`, {
    headers: authHeaders(),
    credentials: 'include',
  })
  if (!res.ok) {
    const err: ApiError = { message: `HTTP ${res.status}`, status: res.status }
    throw err
  }
  return res.blob()
}
