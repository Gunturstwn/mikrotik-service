# Progress Tracker

Update file ini setelah setiap perubahan implementasi yang meaningful.

## Current Phase

- Complete (Landing Page + Login Portal)

## Current Goal

- Maintenance dan enhancement landing page dan auth flow

## Completed

- Setup project Vue 3 + TypeScript + Vite 6
- Design system lengkap di `main.css` (CSS Custom Properties, reset, utilities)
- Konfigurasi konten terpusat di `config/company.ts`
- Router setup (Landing Page + Login Page)
- AppNavbar dengan scroll detection, mobile hamburger, login/logout button
- HeroSection dengan animated background (grid, orbs), CTA buttons, feature pills
- StatsSection dengan animated counters (IntersectionObserver)
- AboutSection dengan profil perusahaan, visi, dan misi
- ServicesSection dengan grid 8 layanan
- CoverageSection dengan peta Indonesia SVG dan info operator
- PartnersSection dengan grid logo partner
- ContactSection dengan info kontak dan form (simulated submit)
- AppFooter dengan navigasi, social media, dan badges sertifikasi
- BackToTop floating button
- LoginPage dengan login form, forgot password flow, CAPTCHA placeholder
- Auth API layer (`api/auth.ts`) dengan typed functions dan error handling
- Token storage helpers (localStorage)
- Scroll-triggered reveal animations
- Responsive design (desktop, tablet, mobile)

## In Progress

- Tidak ada

## Next Up

- Integrasi form kontak dengan backend API
- Dashboard pelanggan setelah login
- Integrasi Cloudflare Turnstile CAPTCHA
- Protected route guard untuk halaman yang memerlukan auth
- SEO optimization (meta tags, structured data)

## Open Questions

- Apakah form kontak akan dikirim ke backend API atau email service?
- Apakah perlu halaman registrasi pelanggan baru?
- Apakah dashboard pelanggan akan di-build di frontend ini atau sebagai app terpisah?
- Apakah perlu multi-language support (EN/ID)?

## Architecture Decisions

- **No state management library**: Skala aplikasi saat ini tidak memerlukan Pinia/Vuex. State lokal per komponen sudah cukup.
- **No UI component library**: Custom CSS dengan design system sendiri untuk kontrol penuh atas visual dan performa.
- **Content config pattern**: Semua konten di satu file (`company.ts`) agar mudah diubah tanpa menyentuh komponen.
- **localStorage untuk auth**: Simple dan sufficient untuk SPA. Jika nanti butuh SSR, perlu migrasi ke cookie-based.
- **Inline SVG icons**: Tidak pakai icon library untuk mengurangi bundle size dan dependency.

## Session Notes

- Backend API berjalan di port 5150 (Rust/Axum)
- Frontend dev server di port 5173 (Vite default)
- Perlu setup CORS di backend untuk development
- `VITE_API_URL` environment variable untuk konfigurasi base URL API
