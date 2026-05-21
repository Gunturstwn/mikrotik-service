# PC24Telin Frontend

## Overview

Landing page dan portal login pelanggan untuk PT PC24 Telekomunikasi Indonesia (PC24Telin). Aplikasi ini berfungsi sebagai company profile digital sekaligus entry point ke portal pelanggan (My PC24). Target pengguna adalah calon klien korporat, instansi pemerintah, dan pelanggan existing yang ingin mengakses dashboard mereka.

## Goals

1. Menampilkan profil perusahaan, layanan, dan jangkauan secara profesional dan modern
2. Menyediakan portal login yang aman dengan proteksi brute-force (CAPTCHA, rate limiting, account locking)
3. Memberikan pengalaman visual premium dengan dark theme dan animasi halus
4. Memudahkan calon klien menghubungi perusahaan melalui form kontak
5. Responsif di semua ukuran layar (desktop, tablet, mobile)

## Core User Flow

1. Pengunjung membuka landing page dan melihat hero section
2. Scroll ke bawah untuk melihat statistik, profil, layanan, jangkauan, dan partner
3. Mengisi form kontak untuk konsultasi atau pertanyaan
4. Pelanggan existing klik "My PC24" untuk masuk ke portal login
5. Login dengan email + password, mendapat JWT token
6. Redirect ke dashboard (belum diimplementasi di frontend)

## Features

### Landing Page

- Hero section dengan animated background dan CTA buttons
- Animated counter statistik (klien, kota, uptime, pengalaman)
- Profil perusahaan dengan visi dan misi
- Grid 8 layanan telekomunikasi
- Peta jangkauan Indonesia dengan info interkoneksi operator
- Grid partner teknologi (Cisco, Huawei, MikroTik, dll)
- Form kontak dengan validasi
- Footer dengan navigasi, social media, dan badges sertifikasi

### Authentication

- Login dengan email + password
- Deteksi CAPTCHA requirement (Cloudflare Turnstile placeholder)
- Account/IP blocking detection
- Forgot password flow (kirim email reset)
- Token storage di localStorage
- Auto-redirect jika sudah login

### UX

- Smooth scroll navigation antar section
- Scroll-triggered reveal animations (IntersectionObserver)
- Back-to-top floating button
- Mobile hamburger menu
- Navbar background blur on scroll

## Scope

### In Scope

- Landing page company profile lengkap
- Login dan forgot password flow
- Responsive design (mobile-first)
- Dark theme dengan design system CSS variables
- Integrasi dengan backend API auth endpoints

### Out of Scope

- Dashboard pelanggan setelah login
- Admin panel
- Multi-language (saat ini Bahasa Indonesia only)
- Light mode / theme switching
- E-commerce / pembayaran online
- Real-time chat / live support

## Success Criteria

1. Landing page load dalam < 3 detik pada koneksi 3G
2. Semua section tampil dengan benar di viewport 320px–2560px
3. Login flow berfungsi end-to-end dengan backend API
4. Form kontak mengirim data ke backend (saat ini simulated)
5. `npm run build` berhasil tanpa error TypeScript
