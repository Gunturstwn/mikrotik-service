# Architecture Context

## Stack

| Layer | Technology | Role |
|-------|-----------|------|
| Framework | Vue 3 + TypeScript | SPA framework dengan Composition API |
| Routing | Vue Router 4 | Client-side routing (History mode) |
| Build | Vite 6 | Dev server + production bundler |
| Styling | CSS Custom Properties | Design system tanpa library eksternal |
| API | Fetch API | HTTP client untuk komunikasi backend |
| Auth | JWT + localStorage | Token-based authentication |
| Font | Inter (Google Fonts) | Typography utama |

## System Boundaries

- `src/config/` — Pusat konfigurasi konten. Semua teks, data layanan, kontak, dan metadata perusahaan didefinisikan di sini. Komponen tidak boleh hardcode konten.
- `src/api/` — API layer. Semua komunikasi HTTP ke backend terisolasi di sini. Komponen memanggil fungsi dari folder ini, bukan fetch langsung.
- `src/components/` — UI components. Masing-masing merepresentasikan satu section di landing page. Stateless (data dari config) kecuali untuk UI state lokal.
- `src/pages/` — Page-level components. Digunakan oleh router. Mengkomposisi komponen-komponen section.
- `src/assets/` — Static assets dan global CSS. Design system didefinisikan di `main.css`.

## Storage Model

- **localStorage**: JWT token (`pc24_token`) dan user ID (`pc24_user_id`) untuk session persistence
- **Component state (ref)**: Form data, UI toggles, animation state — tidak perlu global store
- **Config file**: Konten statis perusahaan (tidak dari database, di-bundle saat build)

## Auth and Access Model

- User login via email + password ke `/api/auth/login`
- Backend return JWT token yang disimpan di localStorage
- Token digunakan untuk request ke protected endpoints (belum diimplementasi di frontend)
- Login status dicek via `authStorage.isLoggedIn()` (cek keberadaan token)
- Navbar menampilkan tombol "My PC24" (login) atau "Keluar" (logout) berdasarkan status
- Backend menerapkan rate limiting, CAPTCHA requirement, dan account locking — frontend menampilkan feedback sesuai error code

## Invariants

1. Semua konten landing page harus berasal dari `config/company.ts` — tidak ada hardcoded text di komponen
2. Semua HTTP request ke backend harus melalui fungsi di `src/api/` — tidak ada fetch langsung di komponen
3. Styling menggunakan CSS Custom Properties dari `main.css` — tidak ada hardcoded hex/color values di komponen
4. Setiap komponen menggunakan `<style scoped>` — tidak ada style leak antar komponen
5. Aplikasi hanya dark theme — tidak ada light mode logic
