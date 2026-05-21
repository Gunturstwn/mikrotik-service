# PC24 Telekomunikasi Indonesia — Frontend

## Overview

Landing page dan portal login pelanggan untuk PT PC24 Telekomunikasi Indonesia (PC24Telin), perusahaan penyedia layanan dan infrastruktur telekomunikasi yang berdiri sejak 2012. Aplikasi ini menampilkan profil perusahaan, layanan, jangkauan, partner, dan menyediakan akses login ke portal pelanggan (My PC24).

## Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| Framework | Vue 3 (Composition API + `<script setup>`) | ^3.5.13 |
| Routing | Vue Router 4 (HTML5 History mode) | ^4.5.0 |
| Build Tool | Vite 6 | ^6.3.5 |
| Language | TypeScript 5.7 (strict mode) | ~5.7.0 |
| Type Checking | vue-tsc | ^2.2.8 |
| Styling | Pure CSS + CSS Custom Properties | — |
| Font | Inter (Google Fonts) | — |

## Project Structure

```
frontend/src/
├── main.ts                 # Entry point: Vue app + router setup
├── App.vue                 # Root component (<RouterView />)
├── config/
│   └── company.ts          # Pusat konfigurasi konten landing page
├── api/
│   └── auth.ts             # Auth API service (login, forgot-password, dll)
├── pages/
│   ├── LandingPage.vue     # Halaman utama (assembles semua section)
│   └── LoginPage.vue       # Portal login pelanggan + forgot password
├── components/
│   ├── AppNavbar.vue       # Navbar fixed dengan scroll detection + hamburger mobile
│   ├── HeroSection.vue     # Hero full-viewport dengan animated background
│   ├── StatsSection.vue    # Animated counter (500+ klien, 34 kota, dll)
│   ├── AboutSection.vue    # Profil perusahaan + Visi/Misi
│   ├── ServicesSection.vue # Grid 8 layanan (fiber, wireless, satellite, dll)
│   ├── CoverageSection.vue # Peta Indonesia SVG + info interkoneksi operator
│   ├── PartnersSection.vue # Grid logo partner (Cisco, Huawei, MikroTik, dll)
│   ├── ContactSection.vue  # Info kontak + form kontak
│   ├── AppFooter.vue       # Footer dengan navigasi, social media, badges
│   └── BackToTop.vue       # Floating scroll-to-top button
└── assets/
    └── main.css            # Design system lengkap (CSS variables, reset, utilities)
```

## Getting Started

### Prerequisites

- Node.js >= 18
- npm >= 9

### Installation

```bash
cd frontend
npm install
```

### Development

```bash
npm run dev
```

Aplikasi berjalan di `http://localhost:5173` (default Vite).

### Build Production

```bash
npm run build
```

Output di folder `dist/`.

### Preview Production Build

```bash
npm run preview
```

## Routes

| Path | Component | Deskripsi |
|------|-----------|-----------|
| `/` | LandingPage.vue | Landing page perusahaan |
| `/login` | LoginPage.vue | Portal login pelanggan (My PC24) |

## Environment Variables

| Variable | Default | Deskripsi |
|----------|---------|-----------|
| `VITE_API_URL` | `http://localhost:5150` | Base URL backend API |

## API Endpoints (Auth)

| Method | Endpoint | Deskripsi |
|--------|----------|-----------|
| POST | `/api/auth/login` | Login, return JWT + user_id |
| GET | `/api/auth/login-status?email=` | Cek status CAPTCHA/block |
| POST | `/api/auth/forgot-password` | Kirim link reset password |
| POST | `/api/auth/reset-password` | Reset password dengan token |

## Key Design Decisions

- **Content-driven**: Semua konten landing page terpusat di `config/company.ts`. Komponen hanya render data dari config ini.
- **Dark theme only**: Background gelap (#060b14), aksen cyan/blue gradient, glassmorphism cards.
- **No UI library**: Semua komponen custom-built dengan CSS Custom Properties.
- **No state management**: State lokal per komponen, cukup untuk skala saat ini.
- **Scroll animations**: IntersectionObserver untuk fade-in sections dan animated counters.
- **Token storage**: JWT disimpan di localStorage via `authStorage` helper.
