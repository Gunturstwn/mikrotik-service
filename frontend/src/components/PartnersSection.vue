<script setup lang="ts">
import { company } from '@/config/company'

// Partner logo colors for the placeholder badges
const badgeColors = [
  { bg: 'rgba(6,182,212,0.1)', border: 'rgba(6,182,212,0.25)', text: '#06b6d4' },
  { bg: 'rgba(59,130,246,0.1)', border: 'rgba(59,130,246,0.25)', text: '#3b82f6' },
  { bg: 'rgba(139,92,246,0.1)', border: 'rgba(139,92,246,0.25)', text: '#8b5cf6' },
  { bg: 'rgba(16,185,129,0.1)', border: 'rgba(16,185,129,0.25)', text: '#10b981' },
  { bg: 'rgba(245,158,11,0.1)', border: 'rgba(245,158,11,0.25)', text: '#f59e0b' },
  { bg: 'rgba(239,68,68,0.1)', border: 'rgba(239,68,68,0.25)', text: '#ef4444' },
]
</script>

<template>
  <section id="partners" class="section partners-section">
    <div class="container">
      <div class="section-header">
        <span class="section-label">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"/>
            <circle cx="9" cy="7" r="4"/>
            <path d="M23 21v-2a4 4 0 0 0-3-3.87"/>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
          </svg>
          Partner
        </span>
        <h2 class="section-title">{{ company.partners.title }}</h2>
        <p class="section-subtitle">{{ company.partners.subtitle }}</p>
      </div>

      <!-- Partner Logo Grid -->
      <div class="partners-grid">
        <div
          v-for="(partner, i) in company.partners.list"
          :key="partner.name"
          class="partner-card glass-card"
          :style="{
            '--p-bg': badgeColors[i % badgeColors.length].bg,
            '--p-border': badgeColors[i % badgeColors.length].border,
            '--p-text': badgeColors[i % badgeColors.length].text,
          }"
        >
          <div class="partner-logo">
            <!-- Placeholder logo badge with initials -->
            <div class="partner-initials">
              {{ partner.name.substring(0, 2).toUpperCase() }}
            </div>
          </div>
          <span class="partner-name">{{ partner.name }}</span>
        </div>
      </div>

      <!-- Trust Text -->
      <div class="partners-trust">
        <div class="trust-divider"></div>
        <p class="trust-text">
          Dipercaya oleh lebih dari
          <strong class="trust-highlight">500+ perusahaan</strong>
          di seluruh Indonesia
        </p>
        <div class="trust-divider"></div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.partners-section {
  position: relative;
}

.section-header {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: var(--space-12);
}

.partners-grid {
  display: grid;
  grid-template-columns: repeat(6, 1fr);
  gap: var(--space-5);
  margin-bottom: var(--space-12);
}

.partner-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-3);
  padding: var(--space-8) var(--space-4);
  cursor: pointer;
  transition: all var(--transition-base);
}

.partner-card:hover {
  background: var(--p-bg, rgba(6, 182, 212, 0.08));
  border-color: var(--p-border, rgba(6, 182, 212, 0.3));
}

.partner-logo {
  width: 64px;
  height: 64px;
  border-radius: var(--radius-lg);
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--p-bg, rgba(6, 182, 212, 0.1));
  border: 1.5px solid var(--p-border, rgba(6, 182, 212, 0.2));
  transition: all var(--transition-base);
}

.partner-card:hover .partner-logo {
  transform: scale(1.08);
  box-shadow: 0 4px 20px rgba(6, 182, 212, 0.2);
}

.partner-initials {
  font-size: var(--font-size-xl);
  font-weight: 800;
  color: var(--p-text, var(--color-cyan));
  letter-spacing: -0.02em;
}

.partner-name {
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-text-secondary);
  transition: color var(--transition-base);
}

.partner-card:hover .partner-name {
  color: var(--p-text, var(--color-cyan));
}

/* Trust Section */
.partners-trust {
  display: flex;
  align-items: center;
  gap: var(--space-6);
  justify-content: center;
}

.trust-divider {
  height: 1px;
  flex: 1;
  max-width: 200px;
  background: linear-gradient(90deg, transparent, var(--color-border));
}

.trust-divider:last-child {
  background: linear-gradient(90deg, var(--color-border), transparent);
}

.trust-text {
  font-size: var(--font-size-base);
  color: var(--color-text-muted);
  text-align: center;
  white-space: nowrap;
}

.trust-highlight {
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  font-weight: 700;
}

@media (max-width: 1100px) {
  .partners-grid { grid-template-columns: repeat(3, 1fr); }
}

@media (max-width: 640px) {
  .partners-grid { grid-template-columns: repeat(2, 1fr); }
  .partners-trust { flex-direction: column; }
  .trust-divider { width: 100px; max-width: 100px; }
  .trust-text { white-space: normal; }
}
</style>
