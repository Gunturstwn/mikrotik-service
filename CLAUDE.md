# CLAUDE.md — Panduan Utama Pengembangan MikroTik Service

> Dokumen ini adalah panduan lengkap untuk memahami, menjalankan, dan mengembangkan proyek **MikroTik Service**. Dibuka kapan saja, pengembang langsung bisa melanjutkan tanpa referensi tambahan.

---

## Daftar Isi

1. [Gambaran Umum Proyek](#1-gambaran-umum-proyek)
2. [Arsitektur: Clean Architecture](#2-arsitektur-clean-architecture)
3. [Struktur Direktori Lengkap](#3-struktur-direktori-lengkap)
4. [Penjelasan Setiap Modul Utama](#4-penjelasan-setiap-modul-utama)
5. [Skema Database Lengkap](#5-skema-database-lengkap)
6. [Setup Environment & Menjalankan Aplikasi](#6-setup-environment--menjalankan-aplikasi)
7. [Migrasi Database (SeaORM)](#7-migrasi-database-seaorm)
8. [Seed Database](#8-seed-database)
9. [Dokumentasi Swagger & Aturan Update API](#9-dokumentasi-swagger--aturan-update-api)
10. [Alur Keamanan (Security Flow)](#10-alur-keamanan-security-flow)
11. [Background Workers](#11-background-workers)
12. [Panduan Menambah Fitur Baru](#12-panduan-menambah-fitur-baru)

---

## 1. Gambaran Umum Proyek

**MikroTik Service** adalah backend REST API berbasis Rust (Axum) untuk sistem billing dan manajemen perangkat MikroTik. Proyek ini menggunakan:

| Komponen          | Teknologi                          |
|-------------------|------------------------------------|
| Web Framework     | Axum 0.7                           |
| ORM / Database    | SeaORM 1.0 + PostgreSQL            |
| Cache             | Redis (via `deadpool-redis`)        |
| Message Queue     | RabbitMQ (via `lapin`)             |
| Object Storage    | MinIO / AWS S3 (via `aws-sdk-s3`)  |
| Auth              | JWT (via `jsonwebtoken`)           |
| Enkripsi Data     | AES-256-GCM (via `aes-gcm`)        |
| Hash Password     | bcrypt (via `bcrypt`)              |
| CAPTCHA           | Cloudflare Turnstile               |
| Email             | SMTP (via `lettre`)                |
| API Docs          | OpenAPI 3.0 / Swagger UI (`utoipa`) |
| Logging/Tracing   | `tracing` + `tracing-subscriber`   |

### Fitur Utama

- **Autentikasi** JWT dengan proteksi brute-force berlapis (rate limit, CAPTCHA, lockout)
- **Manajemen User** dengan RBAC (Role-Based Access Control)
- **Manajemen Perangkat MikroTik** (CRUD, koneksi pool, enkripsi kredensial)
- **Monitoring Real-time**: System resource, interface stats, SSE traffic monitor, torch
- **Config Snapshot**: Backup konfigurasi RouterOS secara berkala dengan diff viewer
- **Audit Trail**: Setiap aksi user dicatat di `audit_logs`
- **Export Data**: CSV & Excel untuk daftar user
- **Upload Foto**: Auto-resize & convert ke WebP via MinIO

---

## 2. Arsitektur: Clean Architecture

Proyek ini mengikuti prinsip **Clean Architecture** dengan pemisahan lapisan yang jelas. Walaupun tidak menggunakan kata "repository" secara eksplisit, pola ketergantungannya identik.

```
┌──────────────────────────────────────────────────────────────┐
│                        HTTP Request                          │
└─────────────────────────────┬────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│              LAYER 1: MIDDLEWARE (Cross-cutting)             │
│  middlewares/auth.rs    — JWT extraction & user context      │
│  middlewares/rate_limit.rs — Token bucket rate limiting      │
└─────────────────────────────┬────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│              LAYER 2: HANDLER / CONTROLLER                   │
│  handlers/auth_handler.rs   — Endpoint autentikasi          │
│  handlers/user_handler.rs   — Endpoint manajemen user       │
│  handlers/mikrotik_handler.rs — Endpoint perangkat MikroTik │
│  handlers/export_handler.rs — Endpoint ekspor data          │
│  handlers/health_handler.rs — Health check endpoint         │
│                                                              │
│  Tanggung jawab:                                             │
│  • Parse request (JSON, multipart, path params, query)       │
│  • Validasi input dasar                                      │
│  • Panggil Service layer                                     │
│  • Tulis Audit Log                                           │
│  • Serialize response                                        │
└─────────────────────────────┬────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│              LAYER 3: SERVICE / USE CASE                     │
│  services/auth_service.rs      — Logika register/login       │
│  services/user_service.rs      — Logika profil user         │
│  services/mikrotik_service.rs  — Logika koneksi RouterOS    │
│  services/security_service.rs  — Brute-force & lockout      │
│  services/captcha_service.rs   — Verifikasi Turnstile CAPTCHA│
│  services/permission_service.rs — Query RBAC permissions    │
│  services/storage_service.rs   — Upload foto ke MinIO       │
│  services/audit/audit_service.rs — Pencatatan audit trail   │
│                                                              │
│  Tanggung jawab:                                             │
│  • Business logic utama aplikasi                             │
│  • Koordinasi antar resource (DB, Redis, RabbitMQ, S3)       │
│  • Tidak tahu apa-apa tentang HTTP request/response          │
└─────────────────────────────┬────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│              LAYER 4: MODEL / ENTITY                         │
│  models/users.rs               — Tabel users                 │
│  models/roles.rs               — Tabel roles                 │
│  models/user_roles.rs          — Tabel pivot user ↔ role    │
│  models/permissions.rs         — Tabel permissions          │
│  models/role_permissions.rs    — Tabel pivot role ↔ perm    │
│  models/mikrotik_clients.rs    — Tabel mikrotik_clients      │
│  models/mikrotik_config_snapshots.rs — Config snapshots     │
│  models/interface_metrics.rs   — Metrik interface history   │
│  models/audit_logs.rs          — Tabel audit_logs           │
│                                                              │
│  Tanggung jawab:                                             │
│  • Definisi struct SeaORM (DeriveEntityModel)                │
│  • Mapping kolom DB ke Rust struct                           │
│  • Relasi antar tabel (has_many, belongs_to)                 │
│  • Method helper enkripsi/dekripsi (mikrotik_clients)        │
└─────────────────────────────┬────────────────────────────────┘
                              │
                              ▼
┌──────────────────────────────────────────────────────────────┐
│              LAYER 5: INFRASTRUKTUR                          │
│  config/database.rs     — Pool koneksi PostgreSQL            │
│  config/redis.rs        — Pool koneksi Redis                 │
│  config/rabbitmq.rs     — Koneksi RabbitMQ                  │
│  config/storage.rs      — Client MinIO/S3                   │
│  config/auth.rs         — JWT create/verify                  │
│  config/mikrotik.rs     — Koneksi ke RouterOS API            │
│  cache/redis_client.rs  — Wrapper operasi Redis              │
│  queue/rabbitmq_client.rs — Wrapper publish RabbitMQ        │
│  pool/mikrotik_pool.rs  — Connection pool perangkat MikroTik │
└──────────────────────────────────────────────────────────────┘
```

### Alur Interaksi Antar Layer

```
Request → Middleware → Handler → Service → Model (SeaORM query) → DB
                                        → RedisClient → Redis
                                        → RabbitMQClient → RabbitMQ
                                        → StorageService → MinIO
                                        → MikrotikPool → MikroTik Device
```

### DTO (Data Transfer Objects)

Layer DTO memisahkan representasi data HTTP dari model database:

```
dto/auth.rs    — RegisterRequest, LoginRequest, AuthResponse, ...
dto/user.rs    — UserProfileResponse, UpdateUserRequest, ...
dto/mikrotik.rs — MikrotikClientRequest, MikrotikClientResponse, ...
```

**Pola:** Request DTO masuk dari HTTP → diproses Service → Response DTO keluar ke HTTP. **Model database tidak pernah dikembalikan langsung ke client.**

---

## 3. Struktur Direktori Lengkap

```
mikrotik-service/
├── Cargo.toml                    # Workspace root + dependencies
├── Cargo.lock
├── CLAUDE.md                     # Dokumen ini
├── .env                          # Variabel environment (JANGAN commit!)
├── .env.example                  # Template .env
│
├── migration/                    # Crate migrasi SeaORM (binary terpisah)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                # Daftar semua migrasi (Migrator)
│       ├── main.rs               # Entry point CLI migrasi
│       ├── m20260403_000001_create_initial_tables.rs
│       ├── m20260404_172022_create_audit_logs_table.rs
│       ├── m20260404_172031_add_security_columns_to_users.rs
│       ├── m20260404_172033_create_export_jobs_table.rs
│       ├── m20260405_000001_create_permissions_tables.rs
│       ├── m20260405_101500_create_mikrotik_clients_table.rs
│       ├── m20260406_132000_encrypt_ssh_port.rs
│       ├── m20260407_150000_create_interface_metrics.rs
│       └── m20260407_160000_create_config_snapshots.rs
│
└── src/
    ├── main.rs                   # Entry point: setup semua service, jalankan server
    ├── lib.rs                    # Re-export semua modul + definisi AppState
    │
    ├── bin/
    │   └── seed.rs               # Binary seed data awal
    │
    ├── config/                   # Konfigurasi koneksi infrastruktur
    │   ├── mod.rs
    │   ├── auth.rs               # JWT create_token / verify_token
    │   ├── database.rs           # PostgreSQL connection pool (SeaORM)
    │   ├── mikrotik.rs           # Koneksi ke RouterOS API port
    │   ├── rabbitmq.rs           # Koneksi RabbitMQ (lapin)
    │   ├── redis.rs              # Pool Redis (deadpool-redis)
    │   └── storage.rs            # Client MinIO/S3 (aws-sdk-s3)
    │
    ├── dto/                      # Data Transfer Objects (request/response shapes)
    │   ├── mod.rs
    │   ├── auth.rs
    │   ├── user.rs
    │   └── mikrotik.rs
    │
    ├── errors/
    │   ├── mod.rs
    │   └── app_error.rs          # Enum AppError + IntoResponse (HTTP status mapping)
    │
    ├── handlers/                 # Controller layer (Axum handler functions)
    │   ├── mod.rs
    │   ├── auth_handler.rs
    │   ├── user_handler.rs
    │   ├── mikrotik_handler.rs
    │   ├── export_handler.rs
    │   └── health_handler.rs
    │
    ├── middlewares/
    │   ├── mod.rs
    │   ├── auth.rs               # UserContext extractor (JWT → user_id + roles)
    │   └── rate_limit.rs         # Global & login rate limit (token bucket via Redis Lua)
    │
    ├── models/                   # SeaORM entity definitions
    │   ├── mod.rs
    │   ├── users.rs
    │   ├── roles.rs
    │   ├── user_roles.rs
    │   ├── permissions.rs
    │   ├── role_permissions.rs
    │   ├── mikrotik_clients.rs   # + method enkripsi/dekripsi AES-GCM
    │   ├── mikrotik_config_snapshots.rs
    │   ├── interface_metrics.rs
    │   └── audit_logs.rs
    │
    ├── routes/                   # Axum Router definitions
    │   ├── mod.rs                # create_router() + CORS config
    │   ├── auth_routes.rs
    │   ├── user_routes.rs
    │   ├── mikrotik_routes.rs
    │   ├── export_routes.rs
    │   └── health.rs
    │
    ├── services/                 # Business logic / Use Case layer
    │   ├── mod.rs
    │   ├── auth_service.rs
    │   ├── user_service.rs
    │   ├── mikrotik_service.rs
    │   ├── security_service.rs
    │   ├── captcha_service.rs
    │   ├── permission_service.rs
    │   ├── storage_service.rs
    │   └── audit/
    │       ├── mod.rs
    │       └── audit_service.rs
    │
    ├── cache/
    │   ├── mod.rs
    │   ├── redis_client.rs       # Wrapper: SET, GET, DEL, INCR, TTL, Lua rate limit
    │   └── user_cache.rs
    │
    ├── queue/
    │   ├── mod.rs
    │   └── rabbitmq_client.rs    # Wrapper: publish ke queue
    │
    ├── workers/                  # Background async tasks
    │   ├── mod.rs
    │   ├── email_worker.rs       # Konsumsi email_queue → kirim via SMTP
    │   └── metrics_worker.rs     # Scrape interface metrics tiap N detik
    │
    ├── pool/
    │   ├── mod.rs
    │   └── mikrotik_pool.rs      # Connection pool ke MikroTik devices (DashMap + TTL)
    │
    ├── export/
    │   ├── mod.rs
    │   ├── csv_exporter.rs       # Export user list ke CSV
    │   └── excel_exporter.rs     # Export user list ke XLSX
    │
    └── utils/
        ├── mod.rs
        ├── aes_gcm.rs            # encrypt() / decrypt() AES-256-GCM + Base64
        ├── encryption.rs         # hash_password() / verify_password() bcrypt
        ├── ip.rs                 # extract_ip() dari headers (X-Forwarded-For, dll)
        └── time.rs
```

---

## 4. Penjelasan Setiap Modul Utama

### 4.1 `AppState` — Shared Application State

Didefinisikan di `src/lib.rs`, di-clone ke setiap handler via Axum `State<AppState>`.

```rust
// src/lib.rs
pub struct AppState {
    pub db: sea_orm::DatabaseConnection,   // Pool koneksi PostgreSQL
    pub redis: cache::RedisClient,          // Wrapper Redis pool
    pub rabbit: queue::RabbitMQClient,     // Publisher RabbitMQ
    pub storage: Arc<aws_sdk_s3::Client>,  // Client MinIO
    pub security: Arc<SecurityService>,    // Brute-force manager
    pub captcha: Arc<CaptchaService>,      // CAPTCHA verifier
    pub mikrotik_pool: Arc<MikrotikPool>,  // Pool koneksi MikroTik
}
```

### 4.2 `config/` — Koneksi Infrastruktur

| File | Fungsi |
|------|--------|
| `database.rs` | `connect()` → SeaORM `DatabaseConnection` (max 100 conn) |
| `redis.rs` | `connect()` → `deadpool_redis::Pool` |
| `rabbitmq.rs` | `connect()` → `lapin::Connection` |
| `storage.rs` | `connect()` → `aws_sdk_s3::Client` dengan path-style URL (MinIO) |
| `auth.rs` | `create_token(user_id, roles)` → JWT, `verify_token(token)` → Claims |
| `mikrotik.rs` | `MikrotikConnection::connect(client, aes_key)` → `MikrotikDevice` |

### 4.3 `middlewares/` — Cross-cutting Concerns

**`auth.rs` — UserContext Extractor:**
```
Request Header "Authorization: Bearer <token>"
    → verify_token() → Claims { sub: Uuid, roles: Vec<String>, exp, iat }
    → DB lookup: pastikan user masih ada dan is_verified = true
    → Ok(UserContext { user_id, roles })
```

Digunakan di handler sebagai parameter `user_ctx: UserContext`.

**`rate_limit.rs`:**
- **Global:** 10 req/s, burst 20 — diterapkan ke semua route via `route_layer`
- **Login:** 2 req/s, burst 5 — diterapkan hanya ke `POST /api/auth/login`
- Implementasi menggunakan **Token Bucket algorithm** via Lua script atomik di Redis

### 4.4 `handlers/` — Controller Layer

Setiap handler mengikuti pola:
```
1. Extract IP (untuk audit)
2. Validasi role (via user_ctx.roles.contains(...))
3. Panggil Service
4. Tulis AuditService::log(...)
5. Return Ok(Json(response)) atau Err(AppError)
```

**Auth Handler endpoints:**
| Method | Path | Deskripsi |
|--------|------|-----------|
| POST | `/api/auth/register` | Daftar user baru (Super Admin only, multipart/form-data) |
| POST | `/api/auth/login` | Login → JWT token |
| GET | `/api/auth/login-status` | Cek status CAPTCHA/block untuk email+IP |
| POST | `/api/auth/verify-token` | Validasi token (Super Admin only) |
| POST | `/api/auth/:id/verify-email` | Verifikasi email user (Super Admin only) |
| POST | `/api/auth/forgot-password` | Kirim reset token via email |
| POST | `/api/auth/reset-password` | Reset password dengan token |

**User Handler endpoints:**
| Method | Path | Deskripsi |
|--------|------|-----------|
| GET | `/api/users/me` | Profil user saat ini |
| PUT | `/api/users/me` | Update profil sendiri (multipart) |
| POST | `/api/users/me/photo` | Upload foto profil |
| GET | `/api/users` | Daftar semua user (Super Admin only, pagination) |
| GET | `/api/users/:id` | Detail user (Super Admin only) |
| PUT | `/api/users/:id` | Update user (Super Admin only, multipart) |
| DELETE | `/api/users/:id` | Soft-delete user (Super Admin only) |

**MikroTik Handler endpoints:**
| Method | Path | Deskripsi |
|--------|------|-----------|
| POST | `/api/mikrotik_client` | Tambah device baru |
| GET | `/api/mikrotik_client` | Daftar semua device |
| GET | `/api/mikrotik_client/:id` | Detail device |
| PUT | `/api/mikrotik_client/:id` | Update device |
| DELETE | `/api/mikrotik_client/:id` | Soft-delete device |
| GET | `/api/mikrotik_client/:id/system/resource/print` | Resource usage (CPU, RAM, HDD) |
| GET | `/api/mikrotik_client/:id/interfaces/print` | Daftar interface + statistik |
| GET | `/api/mikrotik_client/:id/interfaces/monitor` | **SSE** real-time traffic monitor |
| GET | `/api/mikrotik_client/:id/interfaces/torch` | **SSE** traffic analyzer (Torch) |
| GET | `/api/mikrotik_client/:id/config/history` | Riwayat config snapshots |
| GET | `/api/mikrotik_client/:id/config/view/:snapshot_id` | Lihat isi snapshot |
| POST | `/api/mikrotik_client/:id/config/backup-now` | Trigger backup manual |
| GET | `/api/mikrotik_client/:id/config/diff` | Diff dua snapshot |

**Export Handler endpoints:**
| Method | Path | Deskripsi |
|--------|------|-----------|
| GET | `/api/export/users/csv` | Download CSV semua user (Super Admin only) |
| GET | `/api/export/users/xlsx` | Download Excel semua user (Super Admin only) |

### 4.5 `services/` — Business Logic (Use Case Layer)

**`auth_service.rs`:**
- `register(db, rabbit, req)` — Hash password, buat user, assign role, publish welcome email ke RabbitMQ
- `login(db, security, captcha, ip, req)` — Cek lock/CAPTCHA → verifikasi password → reset failure counter → buat JWT
- `forgot_password(db, redis, rabbit, req)` — Generate UUID token, simpan di Redis (TTL 1 jam), publish email
- `reset_password(db, redis, req)` — Lookup token dari Redis → update password → hapus token

**`mikrotik_service.rs`:**
- Semua operasi ke RouterOS dilakukan via `MikrotikPool.get_connection(...)` yang return `Arc<Mutex<MikrotikDevice>>`
- Kredensial (username, password, port) **selalu didekripsi** saat koneksi dibuka
- `monitor_interfaces()` dan `get_torch()` return `impl Stream<...>` untuk **SSE (Server-Sent Events)**
- `perform_versioned_backup()` — Fetch config, hitung SHA-256 hash, simpan ke DB hanya jika hash berbeda

**`security_service.rs`:**
```
Login Failure → track_failure(ip, email):
  • Increment counter di Redis (TTL 24 jam)
  • ≥10 failures → block 15 menit (900s)
  • ≥20 failures → block 1 jam (3600s)
  • ≥30 failures → block 6 jam (21600s)

check_status(ip, email):
  • Cek block key → Blocked(ttl)
  • Cek failure count ≥3 → CaptchaRequired
  • Else → Allowed
```

**`storage_service.rs`:**
```
process_and_upload_image(client, bytes):
  1. Validasi image (load_from_memory - reject non-image)
  2. Resize ke max 800x800 (Lanczos3 filter)
  3. Encode ke WebP
  4. Validasi ukuran < 5MB
  5. Upload ke MinIO bucket
  6. Return public URL
```

### 4.6 `models/` — Entity Layer (SeaORM)

Setiap file berisi struct `Model` dengan derive `DeriveEntityModel`, `Relation` enum, dan `impl ActiveModelBehavior`.

**`mikrotik_clients.rs`** memiliki method khusus:
```rust
// Enkripsi saat simpan ke DB (AES-256-GCM + Base64)
model.set_encrypted_fields(username, password, winbox, api, ftp, ssh, aes_key)?;

// Dekripsi saat dibaca dari DB
let username = model.decrypt_username(aes_key)?;
let port_api = model.decrypt_port_api(aes_key)?;
```

### 4.7 `pool/mikrotik_pool.rs` — Connection Pool

Pool menggunakan `DashMap<Uuid, MikrotikPoolEntry>` (thread-safe HashMap):
- **Lazy Loading**: Koneksi dibuka pertama kali device diakses
- **TTL 30 detik**: Koneksi idle > 30 detik dihapus oleh cleanup task
- **Audit trail**: Setiap connect/disconnect/evict dicatat di `audit_logs`
- `invalidate(device_id)` — Dipanggil setelah update/delete device untuk paksa reconnect

### 4.8 `cache/redis_client.rs`

Wrapper operasi Redis:
```
SET key value EX ttl         → set(key, value, ttl_secs)
GET key                      → get(key) → Option<String>
DEL key                      → del(key)
INCR key + EXPIRE (lazy TTL) → incr(key, ttl_secs)
TTL key                      → ttl(key) → Option<u64>
Lua Token Bucket Script      → check_rate_limit(key, rate, burst)
```

### 4.9 `queue/rabbitmq_client.rs`

Wrapper publish ke RabbitMQ. Setiap publish membuat channel baru (recommended pattern untuk lapin):
```
publish("email_queue", json_payload_str)
```

### 4.10 `utils/`

| File | Fungsi |
|------|--------|
| `aes_gcm.rs` | `encrypt(data, key)` / `decrypt(b64, key)` — AES-256-GCM dengan nonce random 12 byte, output Base64 |
| `encryption.rs` | `hash_password(pwd)` bcrypt / `verify_password(pwd, hash)` |
| `ip.rs` | `extract_ip(headers)` — baca `X-Forwarded-For`, `X-Real-IP`, atau `Host` |
| `time.rs` | Utilitas waktu (jika dibutuhkan) |

---

## 5. Skema Database Lengkap

### ERD (Entity Relationship Diagram)

```
┌─────────────┐        ┌──────────────┐        ┌─────────────┐
│    users    │        │  user_roles  │        │    roles    │
│─────────────│        │──────────────│        │─────────────│
│ id (PK,UUID)│◄──────►│ user_id (FK) │◄──────►│ id (PK,UUID)│
│ name        │        │ role_id (FK) │        │ name        │
│ email (UQ)  │        └──────────────┘        │ created_at  │
│ password    │                                 │ updated_at  │
│ phone       │        ┌──────────────────┐     │ deleted_at  │
│ photo       │        │ role_permissions  │     └──────┬──────┘
│ address     │        │──────────────────│            │
│ lat         │        │ role_id (FK)     │◄───────────┘
│ lng         │        │ permission_id(FK)│
│ is_verified │        └────────┬─────────┘
│ payment_tok │                 │        ┌─────────────────┐
│ created_at  │                 └───────►│   permissions   │
│ updated_at  │                          │─────────────────│
│ deleted_at  │                          │ id (PK,UUID)    │
└──────┬──────┘                          │ name            │
       │                                 │ code (UQ)       │
       │ created_by                      │ created_at      │
       ▼                                 │ updated_at      │
┌──────────────────────┐                 └─────────────────┘
│   mikrotik_clients   │
│──────────────────────│
│ id (PK,UUID)         │
│ name_device          │         ┌───────────────────────────┐
│ host                 │         │ mikrotik_config_snapshots  │
│ username (encrypted) │         │───────────────────────────│
│ password (encrypted) │◄───────►│ id (PK,UUID)              │
│ port_winbox(encr.)   │         │ mikrotik_id (FK)          │
│ port_api (encrypted) │         │ config_content (TEXT)     │
│ port_ftp (encrypted) │         │ config_hash (SHA-256)     │
│ port_ssh (encrypted) │         │ created_at                │
│ location             │         └───────────────────────────┘
│ latitude             │
│ longitude            │         ┌───────────────────────────┐
│ timezone             │         │    interface_metrics       │
│ created_at           │         │───────────────────────────│
│ updated_at           │◄───────►│ id (PK,UUID)              │
│ deleted_at           │         │ mikrotik_id (FK, CASCADE) │
│ created_by (FK→users)│         │ interface_name            │
│ updated_by (FK→users)│         │ rx_byte (BIGINT)          │
│ deleted_by (FK→users)│         │ tx_byte (BIGINT)          │
└──────────────────────┘         │ rx_packet (BIGINT)        │
                                 │ tx_packet (BIGINT)        │
┌───────────────────────┐        │ captured_at               │
│      audit_logs       │        └───────────────────────────┘
│───────────────────────│
│ id (PK,UUID)          │
│ user_id (nullable,UUID│
│ action                │
│ method                │
│ path                  │
│ status (INT)          │
│ ip                    │
│ metadata (JSON)       │
│ created_at            │
└───────────────────────┘
```

### Detail Tabel

#### `roles`
```sql
CREATE TABLE roles (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(50) NOT NULL UNIQUE,
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at  TIMESTAMP
);
-- Default roles: Super Admin, Admin, Finance, Teknisi, Customer
```

#### `users`
```sql
CREATE TABLE users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR NOT NULL,
    email         VARCHAR NOT NULL UNIQUE,
    password      VARCHAR NOT NULL,          -- bcrypt hash
    phone         VARCHAR(20),
    photo         TEXT,                      -- URL ke MinIO
    address       TEXT,
    lat           DECIMAL(10, 8),
    lng           DECIMAL(11, 8),
    is_verified   BOOLEAN NOT NULL DEFAULT false,
    payment_token TEXT,
    created_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at    TIMESTAMP                  -- soft delete
);
```

#### `user_roles`
```sql
CREATE TABLE user_roles (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (user_id, role_id)
);
```

#### `permissions`
```sql
CREATE TABLE permissions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name        VARCHAR(100) NOT NULL,
    code        VARCHAR(100) NOT NULL UNIQUE, -- e.g. "users.list", "device.manage"
    created_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at  TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### `role_permissions`
```sql
CREATE TABLE role_permissions (
    role_id       UUID REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);
```

#### `mikrotik_clients`
```sql
CREATE TABLE mikrotik_clients (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name_device  VARCHAR NOT NULL,
    host         VARCHAR NOT NULL,
    username     VARCHAR NOT NULL,     -- AES-256-GCM encrypted + Base64
    password     VARCHAR NOT NULL,     -- AES-256-GCM encrypted + Base64
    port_winbox  VARCHAR,              -- AES-256-GCM encrypted + Base64
    port_api     VARCHAR,              -- AES-256-GCM encrypted + Base64
    port_ftp     VARCHAR,              -- AES-256-GCM encrypted + Base64
    port_ssh     VARCHAR,              -- AES-256-GCM encrypted + Base64
    location     TEXT,
    latitude     DECIMAL(10, 8),
    longitude    DECIMAL(11, 8),
    timezone     VARCHAR,
    created_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at   TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at   TIMESTAMP,           -- soft delete
    created_by   UUID REFERENCES users(id),
    updated_by   UUID REFERENCES users(id),
    deleted_by   UUID REFERENCES users(id)
);
```

#### `mikrotik_config_snapshots`
```sql
CREATE TABLE mikrotik_config_snapshots (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mikrotik_id    UUID NOT NULL REFERENCES mikrotik_clients(id) ON DELETE CASCADE,
    config_content TEXT NOT NULL,         -- full RouterOS exported config
    config_hash    VARCHAR NOT NULL,      -- SHA-256 untuk deteksi perubahan
    created_at     TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_mikrotik_config_snapshots_client_time
    ON mikrotik_config_snapshots (mikrotik_id, created_at);
```

#### `interface_metrics`
```sql
CREATE TABLE interface_metrics (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    mikrotik_id    UUID NOT NULL REFERENCES mikrotik_clients(id) ON DELETE CASCADE,
    interface_name VARCHAR NOT NULL,
    rx_byte        BIGINT NOT NULL,
    tx_byte        BIGINT NOT NULL,
    rx_packet      BIGINT NOT NULL,
    tx_packet      BIGINT NOT NULL,
    captured_at    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_interface_metrics_mikrotik_interface_time
    ON interface_metrics (mikrotik_id, interface_name, captured_at);
```

#### `audit_logs`
```sql
CREATE TABLE audit_logs (
    id         UUID PRIMARY KEY,
    user_id    UUID,                -- nullable (anonim/sistem)
    action     VARCHAR NOT NULL,   -- e.g. "USER_LOGIN_SUCCESS"
    method     VARCHAR NOT NULL,   -- HTTP method: GET, POST, dll
    path       VARCHAR NOT NULL,   -- URL path
    status     INTEGER NOT NULL,   -- HTTP status code
    ip         VARCHAR NOT NULL,   -- Client IP
    metadata   JSONB,              -- Context tambahan (JSON)
    created_at TIMESTAMP NOT NULL
);
```

### Contoh Kode Aksi Audit yang Direkam

```
USER_REGISTER_SUCCESS / USER_REGISTER_FAILED
USER_LOGIN_SUCCESS / USER_LOGIN_FAILED
USER_GET_PROFILE
USER_PROFILE_UPDATED / USER_PHOTO_UPLOADED
USER_LIST_ACCESSED
USER_DETAIL_ACCESSED
USER_DELETED / USER_UPDATED_BY_ADMIN
USER_EMAIL_VERIFIED
FORGOT_PASSWORD_REQUESTED
PASSWORD_RESET_SUCCESS / PASSWORD_RESET_FAILED
MIKROTIK_CLIENT_READ / MIKROTIK_CONNECTION
MIKROTIK_METRICS_SCRAPE
EXPORT_CSV_SUCCESS / EXPORT_XLSX_SUCCESS
VERIFY_TOKEN_SUCCESS
```

### RBAC: Role → Permission Matrix

| Permission Code      | Super Admin | Admin | Finance | Teknisi | Customer |
|---------------------|:-----------:|:-----:|:-------:|:-------:|:--------:|
| `users.list`        | ✅ | ✅ | ✅ | ✅ | ❌ |
| `users.detail`      | ✅ | ✅ | ❌ | ❌ | ❌ |
| `users.create`      | ✅ | ✅ | ❌ | ❌ | ❌ |
| `users.update`      | ✅ | ✅ | ❌ | ❌ | ❌ |
| `users.delete`      | ✅ | ❌ | ❌ | ❌ | ❌ |
| `users.verify`      | ✅ | ❌ | ❌ | ❌ | ❌ |
| `export.csv`        | ✅ | ❌ | ✅ | ❌ | ❌ |
| `export.xlsx`       | ✅ | ❌ | ✅ | ❌ | ❌ |
| `audit.view`        | ✅ | ❌ | ❌ | ❌ | ❌ |
| `billing.view`      | ✅ | ✅ | ✅ | ❌ | ✅ |
| `billing.create`    | ✅ | ✅ | ✅ | ❌ | ❌ |
| `billing.update`    | ✅ | ✅ | ✅ | ❌ | ❌ |
| `device.view`       | ✅ | ✅ | ❌ | ✅ | ❌ |
| `device.manage`     | ✅ | ✅ | ❌ | ✅ | ❌ |
| `profile.update`    | ✅ | ✅ | ✅ | ✅ | ✅ |
| `profile.photo`     | ✅ | ✅ | ✅ | ✅ | ✅ |

---

## 6. Setup Environment & Menjalankan Aplikasi

### 6.1 Prerequisites

```bash
# 1. Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable

# 2. PostgreSQL (v14+)
# Ubuntu/Debian:
sudo apt install postgresql postgresql-client

# 3. Redis (v7+)
sudo apt install redis-server

# 4. RabbitMQ
sudo apt install rabbitmq-server

# 5. MinIO (untuk development, via Docker)
docker run -d --name minio \
  -p 9000:9000 -p 9001:9001 \
  -e MINIO_ROOT_USER=minioadmin \
  -e MINIO_ROOT_PASSWORD=minioadmin \
  quay.io/minio/minio server /data --console-address ":9001"
```

### 6.2 File `.env`

Buat file `.env` di root proyek (`mikrotik-service/.env`):

```env
# ─── Database ───────────────────────────────────────────────
DATABASE_URL=postgres://postgres:password@localhost:5432/mikrotik_db

# ─── Redis ──────────────────────────────────────────────────
REDIS_URL=redis://127.0.0.1:6379

# ─── RabbitMQ ───────────────────────────────────────────────
RABBITMQ_URL=amqp://guest:guest@localhost:5672/%2F

# ─── MinIO / S3 ─────────────────────────────────────────────
MINIO_ENDPOINT=http://localhost:9000
MINIO_ROOT_USER=minioadmin
MINIO_ROOT_PASSWORD=minioadmin
MINIO_BUCKET=mikrotik-images

# ─── JWT ────────────────────────────────────────────────────
# Generate: openssl rand -hex 32
JWT_SECRET=your_super_secret_jwt_key_min_32_chars

# ─── AES-256 Encryption (untuk kredensial MikroTik) ─────────
# WAJIB 32 bytes atau 64 hex chars
# Generate: openssl rand -hex 32
AES_KEY=your_64_char_hex_key_here_exactly_64_chars_for_aes256

# ─── SMTP (Email) ───────────────────────────────────────────
SMTP_HOST=localhost
SMTP_PORT=1025
SMTP_USER=noreply@example.com
SMTP_PASS=your_smtp_password

# ─── Cloudflare Turnstile CAPTCHA ───────────────────────────
# Gunakan "1x00000000000000000000AA" untuk testing (selalu pass)
TURNSTILE_SECRET_KEY=your_cloudflare_turnstile_secret

# ─── Application ────────────────────────────────────────────
APP_PORT=5150
APP_ENV=development   # Set ke "production" untuk nonaktifkan Swagger UI

# ─── Workers ────────────────────────────────────────────────
METRICS_INTERVAL=60   # Interval scrape metrics dalam detik (default: 60)

# ─── Logging ────────────────────────────────────────────────
RUST_LOG=mikrotik_service=debug,tower_http=debug
```

> **PENTING:** File `.env` berisi secret, **jangan pernah di-commit ke Git**.

### 6.3 Setup Database

```bash
# Buat database PostgreSQL
psql -U postgres -c "CREATE DATABASE mikrotik_db;"

# Aktifkan ekstensi pgcrypto (untuk gen_random_uuid())
psql -U postgres -d mikrotik_db -c "CREATE EXTENSION IF NOT EXISTS pgcrypto;"
```

### 6.4 Jalankan Aplikasi

```bash
# Clone & masuk ke direktori
cd mikrotik-service

# Install dependencies
cargo build

# Jalankan server (migrasi berjalan otomatis saat startup)
cargo run

# Atau dengan logging verbose
RUST_LOG=debug cargo run
```

Server akan berjalan di `http://0.0.0.0:5150`
Swagger UI tersedia di `http://localhost:5150/swagger-ui` (hanya di non-production)

### 6.5 Verifikasi Setup

```bash
# Cek health endpoint
curl http://localhost:5150/api/health

# Response sukses:
# {"status":"ok","database":"connected","redis":"connected","rabbitmq":"connected"}
```

---

## 7. Migrasi Database (SeaORM)

Proyek menggunakan `sea-orm-migration` sebagai sub-crate terpisah di direktori `migration/`.

**Catatan penting:** Migrasi berjalan **otomatis** saat `cargo run` dipanggil di main application via:
```rust
migration::Migrator::up(&db, None).await.expect("Failed to run migrations");
```

### 7.1 Perintah Migrasi via CLI

Semua perintah dijalankan dari direktori `migration/`:

```bash
cd migration/

# Cek status semua migrasi
cargo run -- status

# Jalankan semua migrasi pending
cargo run -- up

# Jalankan N migrasi pending
cargo run -- up -n 3

# Rollback 1 migrasi terakhir
cargo run -- down

# Rollback N migrasi terakhir
cargo run -- down -n 2

# Reset: rollback semua, lalu run semua ulang
cargo run -- refresh

# Drop semua tabel, lalu run semua migrasi
cargo run -- fresh

# Rollback semua
cargo run -- reset
```

### 7.2 Membuat Migrasi Baru

**Langkah 1:** Generate file migrasi baru

```bash
cd migration/

# Format nama: deskripsi singkat tanpa spasi
cargo run -- generate create_billing_table
```

Perintah ini akan membuat file baru di `migration/src/` dengan nama seperti:
`m20260501_143022_create_billing_table.rs`

**Langkah 2:** Edit file migrasi yang baru dibuat

```rust
// migration/src/m20260501_143022_create_billing_table.rs
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(BillingTable::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(BillingTable::Id)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(BillingTable::UserId).uuid().not_null())
                    .col(ColumnDef::new(BillingTable::Amount).decimal_len(15, 2).not_null())
                    .col(ColumnDef::new(BillingTable::Status).string_len(20).not_null().default("pending"))
                    .col(
                        ColumnDef::new(BillingTable::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_billing_user_id")
                            .from(BillingTable::Table, BillingTable::UserId)
                            .to(Alias::new("users"), Alias::new("id"))
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(BillingTable::Table).to_owned())
            .await
    }
}

// Definisi nama tabel dan kolom menggunakan Iden
#[derive(Iden)]
enum BillingTable {
    Table,
    Id,
    UserId,
    Amount,
    Status,
    CreatedAt,
}
```

**Langkah 3:** Daftarkan migrasi di `migration/src/lib.rs`

```rust
// migration/src/lib.rs
mod m20260501_143022_create_billing_table;  // ← tambahkan ini

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            // ... migrasi yang sudah ada ...
            Box::new(m20260501_143022_create_billing_table::Migration),  // ← tambahkan ini
        ]
    }
}
```

**Langkah 4:** Buat SeaORM Entity untuk tabel baru

```rust
// src/models/billing.rs
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "billing_table")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub amount: Decimal,
    pub status: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
```

**Langkah 5:** Tambahkan ke `src/models/mod.rs`

```rust
// src/models/mod.rs
pub mod billing;  // ← tambahkan ini
```

**Langkah 6:** Jalankan migrasi

```bash
# Dari root proyek (akan auto-run saat startup)
cargo run

# Atau manual dari direktori migration
cd migration && cargo run -- up
```

### 7.3 Menambah Kolom ke Tabel Existing

```bash
cd migration/
cargo run -- generate add_description_to_mikrotik_clients
```

```rust
// Di file migrasi yang baru dibuat:
async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Alias::new("mikrotik_clients"))
                .add_column(
                    ColumnDef::new(Alias::new("description"))
                        .text()
                        .null()
                )
                .to_owned(),
        )
        .await
}

async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    manager
        .alter_table(
            Table::alter()
                .table(Alias::new("mikrotik_clients"))
                .drop_column(Alias::new("description"))
                .to_owned(),
        )
        .await
}
```

---

## 8. Seed Database

Seed mengisi data awal: roles, user Super Admin, permissions, dan role-permission assignments.

### 8.1 Menjalankan Seed

```bash
# Dari root proyek
cargo run --bin seed
```

### 8.2 Apa yang Di-seed

```
✅ Roles:
   - Super Admin
   - Admin
   - Finance
   - Teknisi
   - Customer

✅ Super Admin User:
   - Email: gntrstwn19x@gmail.com
   - Password: numbernine9
   - is_verified: true

✅ Permissions (16 permission codes):
   users.list, users.detail, users.create, users.update, users.delete,
   users.verify, export.csv, export.xlsx, audit.view,
   billing.view, billing.create, billing.update,
   device.view, device.manage, profile.update, profile.photo

✅ Role-Permission Assignments (sesuai matrix di atas)
```

> **PENTING UNTUK PRODUCTION:** Sebelum deploy ke production, ubah kredensial Super Admin di `src/bin/seed.rs` atau hapus setelah login pertama dan buat user baru via API.

### 8.3 Seed Bersifat Idempotent

Seed dapat dijalankan berkali-kali tanpa error. Setiap operasi dicek terlebih dahulu (`find` → jika sudah ada, skip; jika belum, insert).

### 8.4 Menambah Data Seed

Untuk menambah seed tambahan, edit `src/bin/seed.rs` dan tambahkan logika setelah bagian yang sudah ada. Contoh menambah user Admin:

```rust
// Tambah di src/bin/seed.rs setelah bagian "Seeding Super Admin User"
println!("Seeding Admin User...");
let admin_email = "admin@example.com";
let existing_admin = users::Entity::find()
    .filter(users::Column::Email.eq(admin_email))
    .one(&db)
    .await
    .unwrap();

let admin_id = match existing_admin {
    Some(u) => u.id,
    None => {
        let id = Uuid::new_v4();
        let new_admin = users::ActiveModel {
            id: Set(id),
            name: Set("Administrator".to_string()),
            email: Set(admin_email.to_string()),
            password: Set(hash_password("admin_password").unwrap()),
            is_verified: Set(true),
            created_at: Set(chrono::Utc::now().naive_utc()),
            updated_at: Set(chrono::Utc::now().naive_utc()),
            ..Default::default()
        };
        new_admin.insert(&db).await.unwrap();
        id
    }
};

// Link ke role Admin
let _ = user_roles::ActiveModel {
    user_id: Set(admin_id),
    role_id: Set(admin_role_id),
    ..Default::default()
}.insert(&db).await;
```

---

## 9. Dokumentasi Swagger & Aturan Update API

### 9.1 Akses Swagger UI

URL: `http://localhost:5150/swagger-ui`

> **Catatan:** Swagger UI **dinonaktifkan di production** (`APP_ENV=production`). Ini adalah fitur keamanan yang disengaja.

### 9.2 Cara Kerja Swagger di Proyek Ini

Proyek menggunakan **utoipa** untuk generate OpenAPI 3.0 spec dari kode Rust. Semua definisi ada di `src/main.rs`:

```rust
#[derive(OpenApi)]
#[openapi(
    paths(
        // Daftarkan SEMUA handler yang ingin muncul di Swagger
        mikrotik_service::handlers::auth_handler::login,
        // ...
    ),
    components(
        schemas(
            // Daftarkan SEMUA DTO schema yang digunakan
            mikrotik_service::dto::auth::LoginRequest,
            // ...
        )
    ),
    modifiers(&SecurityAddon)  // Menambahkan bearer_auth ke security scheme
)]
struct ApiDoc;
```

### 9.3 Aturan Wajib Saat Menambah/Mengubah API

**SETIAP KALI menambah endpoint baru atau mengubah request/response, wajib:**

#### A. Tambahkan `#[utoipa::path(...)]` di atas handler function

```rust
// src/handlers/billing_handler.rs
#[utoipa::path(
    post,                              // HTTP method
    path = "/api/billing",            // URL path (harus sesuai routes)
    request_body = CreateBillingRequest, // DTO request (jika ada body)
    responses(
        (status = 201, description = "Billing created", body = BillingResponse),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    params(
        // Jika ada query params
        ("page" = Option<u64>, Query, description = "Page number"),
    ),
    security(("bearer_auth" = [])),   // Wajib jika endpoint butuh auth
    tag = "Billing"                   // Grup di Swagger UI
)]
pub async fn create_billing(/* ... */) { /* ... */ }
```

#### B. Tambahkan `#[derive(ToSchema)]` di DTO

```rust
// src/dto/billing.rs
use utoipa::ToSchema;

#[derive(Debug, Deserialize, Validate, ToSchema)]
pub struct CreateBillingRequest {
    /// Jumlah tagihan dalam Rupiah
    #[schema(example = 150000.00)]
    pub amount: f64,
    
    /// Keterangan tagihan
    #[schema(example = "Tagihan bulan Januari 2026", nullable = true)]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BillingResponse {
    pub id: Uuid,
    pub amount: f64,
    pub status: String,
    pub created_at: NaiveDateTime,
}
```

#### C. Daftarkan handler dan schema di `src/main.rs`

```rust
// src/main.rs — di dalam #[derive(OpenApi)] #[openapi(...)]
#[openapi(
    paths(
        // ... path yang sudah ada ...
        mikrotik_service::handlers::billing_handler::create_billing,  // ← TAMBAHKAN
        mikrotik_service::handlers::billing_handler::list_billings,   // ← TAMBAHKAN
    ),
    components(
        schemas(
            // ... schema yang sudah ada ...
            mikrotik_service::dto::billing::CreateBillingRequest,  // ← TAMBAHKAN
            mikrotik_service::dto::billing::BillingResponse,       // ← TAMBAHKAN
        )
    ),
    // ...
)]
struct ApiDoc;
```

#### D. Daftarkan di Router

```rust
// src/routes/billing_routes.rs
use axum::{routing::{get, post}, Router};
use crate::AppState;
use crate::handlers::billing_handler;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", post(billing_handler::create_billing))
        .route("/", get(billing_handler::list_billings))
}
```

```rust
// src/routes/mod.rs — tambahkan ke create_router()
mod billing_routes;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        // ... route yang sudah ada ...
        .nest("/api/billing", billing_routes::routes())  // ← TAMBAHKAN
        // ...
}
```

### 9.4 Konvensi Dokumentasi Utoipa

| Atribut | Kapan Digunakan |
|---------|----------------|
| `#[schema(example = "...")]` | Selalu berikan contoh nilai untuk setiap field |
| `#[schema(nullable = true)]` | Untuk field `Option<T>` |
| `#[schema(value_type = String, format = Binary)]` | Untuk field file upload |
| `security(("bearer_auth" = []))` | Setiap endpoint yang memerlukan JWT |
| `tag = "NamaGrup"` | Untuk mengelompokkan endpoint di Swagger UI |

### 9.5 Verifikasi Swagger Setelah Perubahan

```bash
# 1. Compile dan jalankan
cargo run

# 2. Buka browser ke:
# http://localhost:5150/swagger-ui

# 3. Cek endpoint baru muncul di grup yang benar
# 4. Cek request body schema sudah benar
# 5. Coba "Try it out" untuk verifikasi fungsional
```

---

## 10. Alur Keamanan (Security Flow)

### 10.1 Login Flow

```
POST /api/auth/login
    │
    ├─► [Rate Limit] login_rate_limit_middleware
    │       Token Bucket: 2 req/s, burst 5 per IP
    │       → 429 jika exceeded
    │
    ├─► [Rate Limit] global_rate_limit_middleware  
    │       Token Bucket: 10 req/s, burst 20 per IP
    │       → 429 jika exceeded
    │
    ├─► login_handler
    │       ├─► AuthService::login()
    │       │       ├─► SecurityService::check_status(ip, email)
    │       │       │       ├─► Redis: cek "login:block:ip:{ip}" → Blocked(ttl)
    │       │       │       ├─► Redis: cek "login:block:user:{sha256(email)}" → Blocked(ttl)
    │       │       │       ├─► Redis: cek "login:fail:ip:{ip}" >= 3 → CaptchaRequired
    │       │       │       └─► Else → Allowed
    │       │       │
    │       │       ├─► [Jika CaptchaRequired] → verifikasi Cloudflare Turnstile token
    │       │       │
    │       │       ├─► DB: cari user by email
    │       │       ├─► bcrypt: verify password
    │       │       │
    │       │       ├─► [Jika gagal] SecurityService::track_failure(ip, email)
    │       │       │       ├─► Redis INCR "login:fail:ip:{ip}" (TTL 24j)
    │       │       │       ├─► Redis INCR "login:fail:user:{sha256(email)}" (TTL 24j)
    │       │       │       ├─► ≥10 failures → SET "login:block:{ip|user}" TTL 900s
    │       │       │       ├─► ≥20 failures → SET "login:block:{ip|user}" TTL 3600s
    │       │       │       └─► ≥30 failures → SET "login:block:{ip|user}" TTL 21600s
    │       │       │
    │       │       ├─► [Jika berhasil] SecurityService::reset_failures(ip, email)
    │       │       │       └─► Redis DEL fail counters
    │       │       │
    │       │       └─► JWT: create_token(user_id, roles) TTL 24 jam
    │       │
    │       └─► AuditService::log("USER_LOGIN_SUCCESS/FAILED")
    │
    └─► Response: AuthResponse { token, user_id }
```

### 10.2 Authenticated Request Flow

```
Request dengan "Authorization: Bearer <token>"
    │
    ├─► UserContext::from_request_parts()
    │       ├─► Parse header "Authorization: Bearer ..."
    │       ├─► verify_token(token) → Claims { sub (UUID), roles, exp }
    │       │       → 401 jika expired/invalid
    │       │
    │       ├─► DB: find_by_id(claims.sub) 
    │       │       → 401 jika user tidak ditemukan
    │       │
    │       ├─► Cek deleted_at IS NULL          ← [FIXED]
    │       │       → 401 "Account has been deleted" jika user di-soft-delete
    │       │
    │       └─► Cek is_verified == true
    │               → 403 jika tidak terverifikasi
    │
    └─► Handler menerima UserContext { user_id, roles }
            └─► Manual role check: user_ctx.roles.contains("Super Admin")
```

### 10.3 MikroTik Credential Security

```
Saat SIMPAN ke DB:
  plaintext credential → AES-256-GCM encrypt → Base64 encode → stored in DB

Saat BACA dari DB:
  stored string → Base64 decode → AES-256-GCM decrypt → plaintext credential
  → digunakan langsung, TIDAK pernah dikembalikan ke client dalam response
```

**Format enkripsi:**
```
[12 bytes nonce random][ciphertext AES-256-GCM] → Base64 STANDARD
```

---

## 11. Background Workers

### 11.1 EmailWorker

**File:** `src/workers/email_worker.rs`

```
RabbitMQ "email_queue" → EmailWorker::run()
    └─► Konsumsi pesan JSON: { "to": "...", "subject": "...", "body": "..." }
    └─► Send via SMTP (lettre)
    └─► ACK setelah berhasil dikirim
```

**Trigger oleh:**
- Register user baru → welcome email
- Forgot password → reset token email

### 11.2 MetricsWorker

**File:** `src/workers/metrics_worker.rs`

```
Loop setiap METRICS_INTERVAL detik (default: 60s):
    └─► Query semua mikrotik_clients yang tidak deleted
    └─► Untuk setiap device (spawn parallel):
        └─► MikrotikService::get_interfaces() via connection pool
        └─► Insert ke interface_metrics:
            { id, mikrotik_id, interface_name, rx_byte, tx_byte, rx_packet, tx_packet, captured_at }
        └─► AuditService::log("MIKROTIK_METRICS_SCRAPE")
```

Data dari `interface_metrics` adalah time-series yang bisa dipakai untuk grafik bandwidth.

### 11.3 MikrotikPool Cleanup Task

**File:** `src/pool/mikrotik_pool.rs`

```
Loop setiap 60 detik:
    └─► Iterasi semua entry di DashMap pool
    └─► Jika last_used > 30 detik yang lalu:
        └─► Remove dari pool (paksa reconnect berikutnya)
        └─► AuditService::log("MIKROTIK_CONNECTION" EXPIRED/EXPIRED_UNUSED)
```

---

## 12. Panduan Menambah Fitur Baru

### Checklist Fitur Baru

```
□ 1. Buat migrasi DB baru (jika perlu tabel/kolom baru)
     cd migration && cargo run -- generate nama_migrasi
     
□ 2. Daftarkan migrasi di migration/src/lib.rs

□ 3. Buat SeaORM entity di src/models/nama.rs
     + tambahkan ke src/models/mod.rs

□ 4. Buat DTO di src/dto/nama.rs (dengan #[derive(ToSchema)])
     + tambahkan ke src/dto/mod.rs

□ 5. Buat service di src/services/nama_service.rs
     + tambahkan ke src/services/mod.rs

□ 6. Buat handler di src/handlers/nama_handler.rs
     + tambahkan ke src/handlers/mod.rs
     + WAJIB tambahkan #[utoipa::path(...)] di setiap handler

□ 7. Buat routes di src/routes/nama_routes.rs
     + tambahkan ke src/routes/mod.rs (create_router())

□ 8. Update src/main.rs:
     + Tambahkan paths(...) di #[openapi] macro
     + Tambahkan schemas(...) di #[openapi] macro

□ 9. Jalankan cargo build dan pastikan compile
□ 10. Jalankan cargo run dan cek Swagger UI
□ 11. Test via Swagger UI "Try it out"
```

### Konvensi Penamaan

| Jenis | Format | Contoh |
|-------|--------|--------|
| Tabel DB | `snake_case` plural | `billing_invoices` |
| Model Rust | `PascalCase` | `BillingInvoice` |
| Service | `NamaService` | `BillingService` |
| Handler | `nama_action` | `create_billing`, `list_billings` |
| Route path | `/api/kebab-case` | `/api/billing-invoices` |
| Audit action | `SUBJECT_ACTION` | `BILLING_CREATED` |
| Permission code | `resource.action` | `billing.create` |

### Error Handling Pattern

```rust
// Service layer: gunakan Result<T, AppError>
pub async fn create_billing(db: &DatabaseConnection, req: CreateBillingRequest) -> Result<BillingResponse, AppError> {
    // Gunakan ? untuk propagate error otomatis
    let result = BillingEntity::find()
        .filter(/* ... */)
        .one(db)
        .await?;  // DbErr otomatis dikonversi ke AppError::DatabaseError
    
    let item = result.ok_or_else(|| AppError::NotFound("Billing not found".to_string()))?;
    
    Ok(BillingResponse { /* ... */ })
}

// Handler layer: return Err(AppError::...) untuk HTTP error
pub async fn create_billing_handler(/* ... */) -> Result<Json<BillingResponse>, AppError> {
    if !user_ctx.roles.contains(&"Finance".to_string()) {
        return Err(AppError::Forbidden("Finance role required".to_string()));
    }
    // ...
}
```

### AppError → HTTP Status Mapping

| AppError Variant | HTTP Status |
|-----------------|-------------|
| `DatabaseError` | 500 |
| `RedisError` | 500 |
| `RabbitMQError` | 500 |
| `StorageError` | 500 |
| `BadRequest(msg)` | 400 |
| `Unauthorized(msg)` | 401 |
| `Forbidden(msg)` | 403 |
| `NotFound(msg)` | 404 |
| `TooManyRequests(msg)` | 429 |
| `InternalServerError(msg)` | 500 |

---

## Ringkasan Cepat (Quick Reference)

```bash
# Jalankan server
cargo run

# Jalankan seed
cargo run --bin seed

# Migrasi: lihat status
cd migration && cargo run -- status

# Migrasi: buat baru
cd migration && cargo run -- generate nama_migrasi

# Migrasi: rollback 1
cd migration && cargo run -- down

# Swagger UI (development only)
http://localhost:5150/swagger-ui

# Health check
curl http://localhost:5150/api/health

# Login untuk dapat JWT
curl -X POST http://localhost:5150/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"gntrstwn19x@gmail.com","password":"numbernine9"}'
```

---

## 13. Known Fixes & Bugfix History

Section ini mendokumentasikan bug yang telah diperbaiki agar tidak terulang.

### 13.1 User CRUD Fixes (2026-05-15)

#### 🔴 Fix #1 — `middlewares/auth.rs`: Soft-deleted user bisa akses API

**Root cause:** Middleware hanya cek `is_verified`, tidak cek `deleted_at`.

**Fix:** Tambahkan pengecekan `deleted_at.is_some()` sebelum `is_verified`:
```rust
if user.deleted_at.is_some() {
    return Err(AppError::Unauthorized("Account has been deleted".to_string()));
}
```

#### 🔴 Fix #3 — `handlers/user_handler.rs`: Upload ke MinIO sebelum cek role

**Root cause:** Di `update_user`, `parse_multipart_update()` (yang upload foto) dipanggil sebelum validasi role Super Admin.

**Fix:** Pindahkan role check ke atas, **sebelum** `parse_multipart_update()`.

#### 🟠 Fix #4 — `services/user_service.rs`: N+1 Query di `find_all`

**Root cause:** `resolve_roles()` dipanggil per-user dalam loop → 2 query DB × N user.

**Fix:** Refactor `find_all` menggunakan batch query:
- 1 query untuk semua `user_roles` (filter `UserId.is_in(user_ids)`)
- 1 query untuk semua `roles` (filter `Id.is_in(all_role_ids)`)
- Build `HashMap<Uuid, Vec<String>>` untuk assign roles ke masing-masing user
- **Total: 2 query untuk semua user**, berapapun jumlahnya

#### 🟡 Fix #5 — `services/user_service.rs`: Error type salah (BadRequest vs NotFound)

**Root cause:** `update_profile` dan `soft_delete` menggunakan `AppError::BadRequest` untuk "User not found" — semantik HTTP yang salah (400 vs 404).

**Fix:** Ganti ke `AppError::NotFound`.

#### 🟡 Fix #6 — `services/user_service.rs`: Double-delete tidak terdeteksi

**Root cause:** `soft_delete` tidak cek apakah `deleted_at` sudah terisi, sehingga delete berulang selalu return 200.

**Fix:** Tambahkan cek `if user_model.deleted_at.is_some()` → return `AppError::NotFound`.

#### 🟡 Fix #7 — `handlers/user_handler.rs`: Error multipart di-swallow diam-diam

**Root cause:** `upload_photo` menggunakan `.unwrap_or(None)` pada `next_field().await`, menyebabkan error koneksi/payload corrupt diabaikan.

**Fix:** Ganti ke `.map_err(|e| AppError::BadRequest(...))?` agar error di-propagate dengan benar.

#### 🟡 Fix #8 — `services/user_service.rs`: Hardcode fallback role `"Customer"`

**Root cause:** `resolve_roles` otomatis return `vec!["Customer"]` jika user tidak punya entry di `user_roles` — inkonsisten dengan data DB.

**Fix:** Return `vec![]` (kosong) dan log `tracing::warn!`. Ini lebih honest — jika user tidak punya role, jangan asumsikan rolenya.

#### 🟡 Fix #2 — `routes/user_routes.rs`: Route definition tidak idiomatis

**Root cause:** Route `/me` dan `/:id` didefinisikan terpisah per-method, berpotensi conflict di Axum 0.7.

**Fix:** Gabungkan method routing dengan chaining:
```rust
.route("/me", get(get_me).put(update_me))
.route("/:id", get(get_user).put(update_user).delete(delete_user))
```

---

### 13.2 Auth Fixes (2026-05-16)

#### 🔴 Fix #1 — `services/auth_service.rs`: Register tanpa DB Transaction

**Root cause:** `user.insert()` dan `user_role.insert()` dijalankan tanpa transaksi. Jika insert role gagal, user terbuat di DB tapi tanpa role → data corrupt permanen.

**Fix:** Bungkus dengan SeaORM transaction:
```rust
let txn = db.begin().await?;
user.insert(&txn).await?;
user_role.insert(&txn).await?;
txn.commit().await?;  // rollback otomatis jika salah satu gagal
```

#### 🔴 Fix #2 — `services/auth_service.rs`: Soft-deleted user bisa login & dapat JWT

**Root cause:** Query login tidak filter `deleted_at IS NULL`, sehingga user yang di-soft-delete masih bisa login.

**Fix:** Tambahkan filter di query login:
```rust
.filter(crate::models::users::Column::DeletedAt.is_null())
```

#### 🟠 Fix #3 — `handlers/auth_handler.rs`: Register response HTTP 200 bukan 201

**Root cause:** `Ok(Json(res))` default ke 200 OK, padahal Swagger docs mendefinisikan 201 Created.

**Fix:** Return tuple `(StatusCode::CREATED, Json(res))`.

#### 🟠 Fix #4 — `handlers/auth_handler.rs`: Validasi duplikat di register

**Root cause:** Validasi `name` (len 3-50) dan `password` (len min 6) dilakukan dua kali: manual di handler dan via `req.validate()`.

**Fix:** Hapus validasi manual, andalkan sepenuhnya `req.validate()` yang sudah menggunakan `#[validate(length(...))]` di DTO.

#### 🟠 Fix #5 — `services/auth_service.rs`: Hardcode fallback "Customer" di login

**Root cause:** Sama seperti fix di `user_service.rs` — jika user tidak punya role, JWT diisi `["Customer"]` secara palsu.

**Fix:** Hapus fallback, return roles sesuai data DB. Jika kosong, log `tracing::warn!`.

#### 🟡 Fix #6 — `services/auth_service.rs`: `verify_email` tidak validasi state

**Root cause:** Bisa memverifikasi user yang sudah deleted, dan re-verifikasi user yang sudah verified.

**Fix:** Tambahkan pengecekan `deleted_at.is_some()` dan `is_verified` sebelum update.

#### 🟡 Fix #7 — `services/auth_service.rs`: `forgot/reset_password` tidak cek `deleted_at`

**Root cause:** User yang di-soft-delete bisa request reset password dan berhasil (mengaktifkan akun kembali secara tidak langsung).

**Fix:** Tambahkan `.filter(Column::DeletedAt.is_null())` pada query di `forgot_password`. Pada `reset_password`, cek `user.deleted_at.is_some()` setelah lookup, dan hapus token Redis jika user sudah deleted.

#### 🟡 Fix #8 — `dto/auth.rs`: Tidak ada validasi panjang password di `LoginRequest`

**Root cause:** `password` field di `LoginRequest` tidak punya `#[validate]`, sehingga string kosong bisa lolos.

**Fix:**
```rust
#[validate(length(min = 1, message = "Password cannot be empty"))]
pub password: String,
```

#### 🟡 Fix #9 — `dto/auth.rs`: Tidak ada validasi format token di `ResetPasswordRequest`

**Root cause:** `token` field menerima string apapun tanpa validasi panjang/format.

**Fix:**
```rust
#[validate(length(min = 36, max = 36, message = "Invalid token format (must be a valid UUID)"))]
pub token: String,
```

#### 🟡 Fix #10 — `config/auth.rs`: JWT_SECRET dibaca dari env setiap request

**Root cause:** `env::var("JWT_SECRET")` dipanggil di setiap `create_token()` dan `verify_token()` — syscall yang tidak perlu.

**Fix:** Cache menggunakan `std::sync::OnceLock`:
```rust
static JWT_SECRET_CACHE: OnceLock<String> = OnceLock::new();

fn get_jwt_secret() -> &'static str {
    JWT_SECRET_CACHE.get_or_init(|| env::var("JWT_SECRET").expect("JWT_SECRET must be set"))
}
```

---

*Dokumen ini dibuat secara otomatis berdasarkan analisis kode proyek. Update dokumen ini setiap kali ada perubahan arsitektur, API baru, atau perubahan skema database.*
