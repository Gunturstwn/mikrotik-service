# Code Standards

## General

- Keep components single-purpose: satu komponen = satu section/feature
- Fix root causes, jangan layer workarounds
- Jangan mix unrelated concerns dalam satu komponen
- Semua konten teks berasal dari `config/company.ts`, bukan hardcoded di template

## TypeScript

- Strict mode aktif (via `vue-tsc`)
- Gunakan explicit interfaces untuk API request/response types
- Validate unknown external input di API layer sebelum digunakan komponen
- Export types dari `api/` files untuk reuse
- Gunakan `import type` untuk type-only imports

## Vue 3

- Gunakan `<script setup lang="ts">` untuk semua komponen
- Composition API only — tidak ada Options API
- Reactive state via `ref()` dan `computed()`
- Lifecycle hooks: `onMounted`, `onUnmounted`
- Props typing via `defineProps<T>()`
- Event typing via `defineEmits<T>()`
- Gunakan `@/` path alias untuk imports

## Styling

- Gunakan CSS Custom Properties dari `main.css` — tidak ada hardcoded hex values
- Setiap komponen wajib `<style scoped>`
- Ikuti spacing scale: `--space-1` sampai `--space-32`
- Ikuti border radius scale: `--radius-sm` sampai `--radius-full`
- Responsive via media queries di dalam scoped style
- Gunakan utility classes global (`.glass-card`, `.btn-primary`, dll) untuk konsistensi

## API Layer

- Semua HTTP calls terisolasi di `src/api/`
- Gunakan `handleResponse<T>()` helper untuk consistent error handling
- Throw typed `ApiError` untuk error responses
- Base URL dari environment variable (`VITE_API_URL`)
- Jangan expose raw fetch di komponen

## File Organization

- `src/config/` — Konfigurasi konten dan konstanta
- `src/api/` — API service functions dan types
- `src/pages/` — Page-level components (digunakan router)
- `src/components/` — Reusable/section components
- `src/assets/` — Global CSS dan static assets

## Naming Conventions

- Components: PascalCase (`AppNavbar.vue`, `HeroSection.vue`)
- Files: PascalCase untuk Vue, camelCase untuk TS (`auth.ts`, `company.ts`)
- CSS classes: kebab-case (`hero-content`, `glass-card`)
- CSS variables: kebab-case dengan prefix (`--color-`, `--space-`, `--radius-`)
- API functions: camelCase dengan suffix (`loginApi`, `forgotPasswordApi`)
- Interfaces: PascalCase (`LoginRequest`, `AuthResponse`)
