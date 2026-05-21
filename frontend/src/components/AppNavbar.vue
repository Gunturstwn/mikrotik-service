<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { company } from '@/config/company'
import { authStorage } from '@/api/auth'

const router = useRouter()
const isLoggedIn = ref(authStorage.isLoggedIn())

const isScrolled = ref(false)
const isMobileOpen = ref(false)

const navLinks = [
  { label: 'Tentang Kami', href: '#about' },
  { label: 'Layanan', href: '#services' },
  { label: 'Jangkauan', href: '#coverage' },
  { label: 'Partner', href: '#partners' },
  { label: 'Kontak', href: '#contact' },
]

const handleScroll = () => {
  isScrolled.value = window.scrollY > 20
}

const scrollToSection = (href: string) => {
  isMobileOpen.value = false
  const el = document.querySelector(href)
  if (el) el.scrollIntoView({ behavior: 'smooth' })
}

onMounted(() => window.addEventListener('scroll', handleScroll))
onUnmounted(() => window.removeEventListener('scroll', handleScroll))
</script>

<template>
  <header class="navbar" :class="{ scrolled: isScrolled }">
    <div class="container navbar-inner">
      <!-- Logo -->
      <a href="#" class="navbar-logo" @click.prevent="scrollToSection('#top')">
        <div class="logo-icon">
          <svg width="32" height="32" viewBox="0 0 32 32" fill="none">
            <circle cx="16" cy="16" r="16" fill="url(#logoGrad)"/>
            <path d="M8 16 L16 8 L24 16 L16 24 Z" fill="white" opacity="0.9"/>
            <circle cx="16" cy="16" r="4" fill="url(#logoGrad2)"/>
            <defs>
              <linearGradient id="logoGrad" x1="0" y1="0" x2="32" y2="32">
                <stop offset="0%" stop-color="#06b6d4"/>
                <stop offset="100%" stop-color="#3b82f6"/>
              </linearGradient>
              <linearGradient id="logoGrad2" x1="0" y1="0" x2="32" y2="32">
                <stop offset="0%" stop-color="#22d3ee"/>
                <stop offset="100%" stop-color="#60a5fa"/>
              </linearGradient>
            </defs>
          </svg>
        </div>
        <span class="logo-text">{{ company.shortName }}</span>
      </a>

      <!-- Desktop Nav -->
      <nav class="navbar-nav">
        <a
          v-for="link in navLinks"
          :key="link.href"
          :href="link.href"
          class="nav-link"
          @click.prevent="scrollToSection(link.href)"
        >{{ link.label }}</a>
      </nav>

      <!-- CTA Buttons -->
      <div class="navbar-actions">
        <a
          :href="company.ctaSecondary.href"
          class="btn btn-secondary navbar-cta"
          @click.prevent="scrollToSection(company.ctaSecondary.href)"
        >
          Hubungi Kami
        </a>
        <router-link
          v-if="!isLoggedIn"
          to="/login"
          id="navbar-login-btn"
          class="btn btn-primary navbar-cta"
        >
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M15 3h4a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2h-4"/>
            <polyline points="10 17 15 12 10 7"/>
            <line x1="15" y1="12" x2="3" y2="12"/>
          </svg>
          My PC24
        </router-link>
        <router-link
          v-else
          to="/dashboard"
          id="navbar-dashboard-btn"
          class="btn btn-primary navbar-cta"
        >
          Dashboard
        </router-link>
      </div>

      <!-- Hamburger -->
      <button
        class="hamburger"
        :class="{ open: isMobileOpen }"
        @click="isMobileOpen = !isMobileOpen"
        aria-label="Toggle menu"
      >
        <span></span>
        <span></span>
        <span></span>
      </button>
    </div>

    <!-- Mobile Menu -->
    <div class="mobile-menu" :class="{ open: isMobileOpen }">
      <a
        v-for="link in navLinks"
        :key="link.href"
        :href="link.href"
        class="mobile-nav-link"
        @click.prevent="scrollToSection(link.href)"
      >{{ link.label }}</a>
      <a
        :href="company.ctaSecondary.href"
        class="btn btn-secondary mobile-cta"
        @click.prevent="scrollToSection(company.ctaSecondary.href)"
      >
        Hubungi Kami
      </a>
      <router-link
        v-if="!isLoggedIn"
        to="/login"
        class="btn btn-primary mobile-cta"
        @click="isMobileOpen = false"
      >
        My PC24
      </router-link>
      <router-link
        v-else
        to="/dashboard"
        class="btn btn-primary mobile-cta"
        @click="isMobileOpen = false"
      >
        Dashboard
      </router-link>
    </div>
  </header>
</template>

<style scoped>
.navbar {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  height: var(--navbar-height);
  transition: background var(--transition-base), backdrop-filter var(--transition-base), border-color var(--transition-base), box-shadow var(--transition-base);
  border-bottom: 1px solid transparent;
}

.navbar.scrolled {
  background: rgba(6, 11, 20, 0.85);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border-bottom-color: var(--color-border);
  box-shadow: 0 4px 32px rgba(0, 0, 0, 0.4);
}

.navbar-inner {
  display: flex;
  align-items: center;
  height: 100%;
  gap: var(--space-8);
}

.navbar-logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
}

.logo-icon {
  animation: float 6s ease-in-out infinite;
}

.logo-text {
  font-size: var(--font-size-xl);
  font-weight: 800;
  letter-spacing: -0.02em;
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.navbar-nav {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  flex: 1;
  justify-content: center;
}

.nav-link {
  padding: var(--space-2) var(--space-4);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  transition: color var(--transition-base), background var(--transition-base);
}

.nav-link:hover {
  color: var(--color-text-primary);
  background: rgba(255, 255, 255, 0.05);
}

.navbar-cta {
  padding: var(--space-2) var(--space-5);
  font-size: var(--font-size-sm);
  flex-shrink: 0;
}

.navbar-actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
}

/* Hamburger */
.hamburger {
  display: none;
  flex-direction: column;
  gap: 5px;
  width: 32px;
  padding: var(--space-2);
  margin-left: auto;
}

.hamburger span {
  display: block;
  height: 2px;
  background: var(--color-text-primary);
  border-radius: 2px;
  transition: all var(--transition-base);
  transform-origin: center;
}

.hamburger.open span:nth-child(1) { transform: translateY(7px) rotate(45deg); }
.hamburger.open span:nth-child(2) { opacity: 0; transform: scaleX(0); }
.hamburger.open span:nth-child(3) { transform: translateY(-7px) rotate(-45deg); }

/* Mobile Menu */
.mobile-menu {
  display: none;
  flex-direction: column;
  padding: var(--space-4) var(--space-6) var(--space-6);
  background: rgba(6, 11, 20, 0.95);
  backdrop-filter: blur(20px);
  border-top: 1px solid var(--color-border);
  gap: var(--space-2);
  max-height: 0;
  overflow: hidden;
  transition: max-height var(--transition-slow), padding var(--transition-slow);
}

.mobile-menu.open {
  max-height: 400px;
}

.mobile-nav-link {
  padding: var(--space-3) var(--space-4);
  font-size: var(--font-size-base);
  font-weight: 500;
  color: var(--color-text-secondary);
  border-radius: var(--radius-md);
  transition: color var(--transition-base), background var(--transition-base);
}

.mobile-nav-link:hover {
  color: var(--color-text-primary);
  background: rgba(255, 255, 255, 0.05);
}

.mobile-cta {
  margin-top: var(--space-2);
  width: 100%;
}

@media (max-width: 768px) {
  .navbar-nav, .navbar-actions { display: none; }
  .hamburger { display: flex; }
  .mobile-menu { display: flex; }
}
</style>
