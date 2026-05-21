<script setup lang="ts">
import { company } from '@/config/company'

const navLinks = [
  {
    title: 'Tentang Kami',
    links: [
      { label: 'Profil Perusahaan', href: '#about' },
      { label: 'Visi & Misi', href: '#about' },
      { label: 'Meet the Team', href: '#about' },
      { label: 'Izin & Legalitas', href: '#about' },
    ],
  },
  {
    title: 'Layanan',
    links: [
      { label: 'Koneksi Fiber Optik', href: '#services' },
      { label: 'Koneksi Wireless', href: '#services' },
      { label: 'Network Security', href: '#services' },
      { label: 'Kolokasi Data Center', href: '#services' },
    ],
  },
  {
    title: 'Perusahaan',
    links: [
      { label: 'Jangkauan Kami', href: '#coverage' },
      { label: 'Partner Kami', href: '#partners' },
      { label: 'Hubungi Kami', href: '#contact' },
      { label: 'Karir', href: '#contact' },
    ],
  },
]

const scrollToSection = (href: string) => {
  const el = document.querySelector(href)
  if (el) el.scrollIntoView({ behavior: 'smooth' })
}
</script>

<template>
  <footer class="footer">
    <!-- Top gradient line -->
    <div class="footer-top-line"></div>

    <div class="container">
      <!-- Main Footer Content -->
      <div class="footer-grid">
        <!-- Brand Column -->
        <div class="footer-brand">
          <div class="footer-logo">
            <svg width="36" height="36" viewBox="0 0 32 32" fill="none">
              <circle cx="16" cy="16" r="16" fill="url(#footerLogoGrad)"/>
              <path d="M8 16 L16 8 L24 16 L16 24 Z" fill="white" opacity="0.9"/>
              <circle cx="16" cy="16" r="4" fill="url(#footerLogoGrad2)"/>
              <defs>
                <linearGradient id="footerLogoGrad" x1="0" y1="0" x2="32" y2="32">
                  <stop offset="0%" stop-color="#06b6d4"/>
                  <stop offset="100%" stop-color="#3b82f6"/>
                </linearGradient>
                <linearGradient id="footerLogoGrad2" x1="0" y1="0" x2="32" y2="32">
                  <stop offset="0%" stop-color="#22d3ee"/>
                  <stop offset="100%" stop-color="#60a5fa"/>
                </linearGradient>
              </defs>
            </svg>
            <span class="footer-logo-text">{{ company.shortName }}</span>
          </div>

          <p class="footer-tagline">{{ company.footer.tagline }}</p>
          <p class="footer-desc">
            Penyedia layanan dan infrastruktur telekomunikasi terpercaya sejak {{ company.foundedYear }}.
            Mendukung transformasi digital nasional.
          </p>

          <!-- Social Links -->
          <div class="footer-social">
            <a :href="company.social.linkedin" target="_blank" rel="noopener" class="footer-social-btn" aria-label="LinkedIn">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"/>
                <rect x="2" y="9" width="4" height="12"/>
                <circle cx="4" cy="4" r="2"/>
              </svg>
            </a>
            <a :href="company.social.instagram" target="_blank" rel="noopener" class="footer-social-btn" aria-label="Instagram">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="2" width="20" height="20" rx="5" ry="5"/>
                <path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"/>
                <line x1="17.5" y1="6.5" x2="17.51" y2="6.5"/>
              </svg>
            </a>
            <a :href="company.social.twitter" target="_blank" rel="noopener" class="footer-social-btn" aria-label="Twitter/X">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
              </svg>
            </a>
            <a :href="company.social.youtube" target="_blank" rel="noopener" class="footer-social-btn" aria-label="YouTube">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor">
                <path d="M22.54 6.42a2.78 2.78 0 0 0-1.95-1.96C18.88 4 12 4 12 4s-6.88 0-8.59.46a2.78 2.78 0 0 0-1.95 1.96A29 29 0 0 0 1 12a29 29 0 0 0 .46 5.58A2.78 2.78 0 0 0 3.41 19.6C5.12 20 12 20 12 20s6.88 0 8.59-.4a2.78 2.78 0 0 0 1.95-1.95A29 29 0 0 0 23 12a29 29 0 0 0-.46-5.58z"/>
                <polygon points="9.75 15.02 15.5 12 9.75 8.98 9.75 15.02" fill="white"/>
              </svg>
            </a>
          </div>
        </div>

        <!-- Nav Columns -->
        <div
          v-for="col in navLinks"
          :key="col.title"
          class="footer-nav-col"
        >
          <h4 class="footer-col-title">{{ col.title }}</h4>
          <ul class="footer-nav-list">
            <li v-for="link in col.links" :key="link.label">
              <a
                :href="link.href"
                class="footer-nav-link"
                @click.prevent="scrollToSection(link.href)"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" class="link-arrow">
                  <path d="M5 12h14M12 5l7 7-7 7"/>
                </svg>
                {{ link.label }}
              </a>
            </li>
          </ul>
        </div>
      </div>

      <!-- Divider -->
      <div class="footer-divider"></div>

      <!-- Bottom Bar -->
      <div class="footer-bottom">
        <p class="footer-copyright">{{ company.footer.copyright }}</p>

        <div class="footer-badges">
          <span class="footer-badge">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
            ISO 27001
          </span>
          <span class="footer-badge">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <polyline points="20 6 9 17 4 12"/>
            </svg>
            Kominfo Certified
          </span>
          <span class="footer-badge">
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <circle cx="12" cy="12" r="10"/>
              <polyline points="12 6 12 12 16 14"/>
            </svg>
            99.9% Uptime
          </span>
        </div>
      </div>
    </div>
  </footer>
</template>

<style scoped>
.footer {
  background: linear-gradient(180deg, var(--color-bg-secondary) 0%, #04080f 100%);
  border-top: 1px solid var(--color-border);
  position: relative;
}

.footer-top-line {
  height: 2px;
  background: var(--gradient-primary);
  opacity: 0.6;
}

.footer-grid {
  display: grid;
  grid-template-columns: 1.5fr 1fr 1fr 1fr;
  gap: var(--space-12);
  padding: var(--space-16) 0 var(--space-12);
}

/* Brand */
.footer-brand {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.footer-logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  margin-bottom: var(--space-2);
}

.footer-logo-text {
  font-size: var(--font-size-xl);
  font-weight: 800;
  letter-spacing: -0.02em;
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.footer-tagline {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-cyan);
  letter-spacing: 0.02em;
}

.footer-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  line-height: 1.75;
}

.footer-social {
  display: flex;
  gap: var(--space-3);
  margin-top: var(--space-2);
}

.footer-social-btn {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  transition: all var(--transition-base);
}

.footer-social-btn:hover {
  background: rgba(6, 182, 212, 0.1);
  border-color: rgba(6, 182, 212, 0.3);
  color: var(--color-cyan);
  transform: translateY(-2px);
}

/* Nav Columns */
.footer-col-title {
  font-size: var(--font-size-sm);
  font-weight: 700;
  color: var(--color-text-primary);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  margin-bottom: var(--space-5);
}

.footer-nav-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.footer-nav-link {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
  transition: color var(--transition-base);
}

.footer-nav-link .link-arrow {
  opacity: 0;
  transform: translateX(-4px);
  transition: all var(--transition-base);
  flex-shrink: 0;
}

.footer-nav-link:hover {
  color: var(--color-cyan-light);
}

.footer-nav-link:hover .link-arrow {
  opacity: 1;
  transform: translateX(0);
}

/* Divider */
.footer-divider {
  height: 1px;
  background: linear-gradient(90deg, transparent, var(--color-border) 20%, var(--color-border) 80%, transparent);
  margin-bottom: var(--space-6);
}

/* Bottom */
.footer-bottom {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: var(--space-8);
  flex-wrap: wrap;
  gap: var(--space-4);
}

.footer-copyright {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

.footer-badges {
  display: flex;
  gap: var(--space-3);
  flex-wrap: wrap;
}

.footer-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-xs);
  font-weight: 600;
  padding: var(--space-1) var(--space-3);
  background: rgba(6, 182, 212, 0.07);
  border: 1px solid rgba(6, 182, 212, 0.15);
  border-radius: var(--radius-full);
  color: var(--color-text-muted);
  letter-spacing: 0.04em;
}

@media (max-width: 1100px) {
  .footer-grid { grid-template-columns: 1fr 1fr; gap: var(--space-8); }
  .footer-brand { grid-column: span 2; }
}

@media (max-width: 640px) {
  .footer-grid { grid-template-columns: 1fr; gap: var(--space-8); }
  .footer-brand { grid-column: auto; }
  .footer-bottom { flex-direction: column; align-items: flex-start; }
}
</style>
