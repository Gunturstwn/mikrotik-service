<script setup lang="ts">
import { company } from '@/config/company'
</script>

<template>
  <section id="services" class="section services-section">
    <div class="container">
      <!-- Header -->
      <div class="section-header">
        <span class="section-label">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
            <line x1="8" y1="21" x2="16" y2="21"/>
            <line x1="12" y1="17" x2="12" y2="21"/>
          </svg>
          Layanan Kami
        </span>
        <h2 class="section-title">
          Solusi Telekomunikasi<br/>
          <span class="gradient-text">Lengkap & Terpadu</span>
        </h2>
        <p class="section-subtitle">
          Kami menyediakan berbagai layanan infrastruktur telekomunikasi untuk mendukung
          keberlangsungan dan pertumbuhan bisnis Anda.
        </p>
      </div>

      <!-- Services Grid -->
      <div class="services-grid">
        <div
          v-for="(service, i) in company.services"
          :key="service.id"
          class="service-card glass-card"
          :style="{ animationDelay: `${i * 0.08}s` }"
        >
          <div class="service-icon-wrap">
            <span class="service-icon">{{ service.icon }}</span>
            <div class="service-icon-glow"></div>
          </div>
          <h3 class="service-title">{{ service.title }}</h3>
          <p class="service-desc">{{ service.description }}</p>
          <div class="service-link">
            <span>Pelajari</span>
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.services-section {
  position: relative;
}

.section-header {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: var(--space-16);
}

.services-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-5);
}

.service-card {
  padding: var(--space-8) var(--space-6);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  cursor: pointer;
  position: relative;
  overflow: hidden;
}

.service-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 2px;
  background: var(--gradient-primary);
  opacity: 0;
  transition: opacity var(--transition-base);
}

.service-card:hover::before {
  opacity: 1;
}

.service-icon-wrap {
  position: relative;
  width: 56px;
  height: 56px;
  flex-shrink: 0;
}

.service-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  font-size: 1.75rem;
  background: rgba(6, 182, 212, 0.08);
  border: 1px solid rgba(6, 182, 212, 0.15);
  border-radius: var(--radius-lg);
  transition: background var(--transition-base), border-color var(--transition-base), transform var(--transition-base);
}

.service-card:hover .service-icon {
  background: rgba(6, 182, 212, 0.15);
  border-color: rgba(6, 182, 212, 0.35);
  transform: scale(1.08);
}

.service-icon-glow {
  position: absolute;
  inset: -10px;
  background: radial-gradient(circle, rgba(6, 182, 212, 0.2) 0%, transparent 70%);
  opacity: 0;
  pointer-events: none;
  transition: opacity var(--transition-base);
}

.service-card:hover .service-icon-glow {
  opacity: 1;
}

.service-title {
  font-size: var(--font-size-base);
  font-weight: 700;
  color: var(--color-text-primary);
  line-height: 1.3;
}

.service-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.7;
  flex: 1;
}

.service-link {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-sm);
  font-weight: 600;
  color: var(--color-cyan);
  opacity: 0;
  transform: translateY(4px);
  transition: opacity var(--transition-base), transform var(--transition-base);
}

.service-card:hover .service-link {
  opacity: 1;
  transform: translateY(0);
}

@media (max-width: 1100px) {
  .services-grid { grid-template-columns: repeat(3, 1fr); }
}

@media (max-width: 768px) {
  .services-grid { grid-template-columns: repeat(2, 1fr); gap: var(--space-4); }
}

@media (max-width: 480px) {
  .services-grid { grid-template-columns: 1fr; }
}
</style>
