<script setup lang="ts">
import { company } from '@/config/company'
</script>

<template>
  <section id="coverage" class="section coverage-section">
    <!-- Background decoration -->
    <div class="coverage-bg" aria-hidden="true"></div>

    <div class="container">
      <!-- Header -->
      <div class="section-header">
        <span class="section-label">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/>
            <circle cx="12" cy="10" r="3"/>
          </svg>
          Jangkauan
        </span>
        <h2 class="section-title">{{ company.coverage.title }}</h2>
        <p class="section-subtitle">{{ company.coverage.subtitle }}</p>
      </div>

      <!-- Highlights Grid -->
      <div class="highlights-grid">
        <div
          v-for="(h, i) in company.coverage.highlights"
          :key="i"
          class="highlight-card glass-card"
        >
          <span class="highlight-icon">{{ h.icon }}</span>
          <div>
            <div class="highlight-label">{{ h.label }}</div>
            <div class="highlight-desc">{{ h.desc }}</div>
          </div>
        </div>
      </div>

      <!-- Map Visual / Infographic -->
      <div class="map-visual glass-card">
        <div class="map-inner">
          <!-- Indonesia map SVG simplified shape -->
          <div class="map-svg-wrap">
            <svg viewBox="0 0 500 200" class="indonesia-map" xmlns="http://www.w3.org/2000/svg">
              <!-- Simplified Indonesia archipelago shape -->
              <!-- Sumatera -->
              <ellipse cx="120" cy="95" rx="65" ry="22" fill="rgba(6,182,212,0.15)" stroke="rgba(6,182,212,0.4)" stroke-width="1.5" transform="rotate(-20 120 95)"/>
              <!-- Jawa -->
              <ellipse cx="230" cy="130" rx="55" ry="14" fill="rgba(6,182,212,0.2)" stroke="rgba(6,182,212,0.5)" stroke-width="1.5" transform="rotate(-5 230 130)"/>
              <!-- Kalimantan -->
              <ellipse cx="285" cy="80" rx="45" ry="40" fill="rgba(59,130,246,0.15)" stroke="rgba(59,130,246,0.4)" stroke-width="1.5"/>
              <!-- Sulawesi -->
              <path d="M340 65 Q355 50 365 70 Q375 90 360 110 Q350 125 345 105 Q335 90 340 65Z" fill="rgba(6,182,212,0.15)" stroke="rgba(6,182,212,0.4)" stroke-width="1.5"/>
              <!-- Papua -->
              <ellipse cx="440" cy="90" rx="48" ry="32" fill="rgba(139,92,246,0.12)" stroke="rgba(139,92,246,0.35)" stroke-width="1.5"/>
              <!-- Bali & Nusa Tenggara -->
              <ellipse cx="290" cy="148" rx="25" ry="10" fill="rgba(16,185,129,0.12)" stroke="rgba(16,185,129,0.35)" stroke-width="1.5" transform="rotate(-10 290 148)"/>
              <!-- Connection lines -->
              <line x1="120" y1="95" x2="230" y2="130" stroke="rgba(6,182,212,0.2)" stroke-width="1" stroke-dasharray="4 4"/>
              <line x1="230" y1="130" x2="285" y2="80" stroke="rgba(6,182,212,0.2)" stroke-width="1" stroke-dasharray="4 4"/>
              <line x1="285" y1="80" x2="355" y2="80" stroke="rgba(6,182,212,0.2)" stroke-width="1" stroke-dasharray="4 4"/>
              <line x1="355" y1="80" x2="440" y2="90" stroke="rgba(6,182,212,0.2)" stroke-width="1" stroke-dasharray="4 4"/>
              <!-- Pulse dots on major cities -->
              <circle cx="230" cy="125" r="4" fill="var(--color-cyan)" opacity="0.9"/>
              <circle cx="230" cy="125" r="10" fill="none" stroke="var(--color-cyan)" stroke-width="1" opacity="0.4" class="pulse-circle"/>
              <circle cx="200" cy="100" r="3" fill="var(--color-blue)" opacity="0.9"/>
              <circle cx="285" cy="75" r="3" fill="var(--color-blue)" opacity="0.9"/>
              <circle cx="355" cy="80" r="3" fill="var(--color-cyan)" opacity="0.9"/>
              <circle cx="120" cy="95" r="3" fill="var(--color-cyan)" opacity="0.9"/>
            </svg>
          </div>

          <div class="map-info">
            <h3 class="map-info-title">Interkoneksi dengan Operator Terkemuka</h3>
            <p class="map-info-sub">Terhubung langsung dengan backbone operator nasional untuk kualitas terbaik.</p>
            <div class="operator-chips">
              <span
                v-for="op in company.coverage.operators"
                :key="op"
                class="operator-chip"
              >{{ op }}</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.coverage-section {
  position: relative;
  overflow: hidden;
}

.coverage-bg {
  position: absolute;
  inset: 0;
  background:
    radial-gradient(ellipse at 20% 50%, rgba(59, 130, 246, 0.06) 0%, transparent 50%),
    radial-gradient(ellipse at 80% 50%, rgba(6, 182, 212, 0.06) 0%, transparent 50%);
  pointer-events: none;
}

.section-header {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: var(--space-12);
}

/* Highlights */
.highlights-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-5);
  margin-bottom: var(--space-8);
}

.highlight-card {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-6);
}

.highlight-icon {
  font-size: 2rem;
  flex-shrink: 0;
  width: 52px;
  height: 52px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(6, 182, 212, 0.08);
  border-radius: var(--radius-md);
  border: 1px solid rgba(6, 182, 212, 0.15);
}

.highlight-label {
  font-size: var(--font-size-base);
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: 2px;
}

.highlight-desc {
  font-size: var(--font-size-sm);
  color: var(--color-text-muted);
}

/* Map Visual */
.map-visual {
  padding: 0;
  overflow: hidden;
}

.map-inner {
  display: grid;
  grid-template-columns: 1fr auto;
  gap: 0;
  min-height: 260px;
}

.map-svg-wrap {
  padding: var(--space-8);
  display: flex;
  align-items: center;
  border-right: 1px solid var(--color-border);
  background: rgba(0, 0, 0, 0.2);
}

.indonesia-map {
  width: 100%;
  max-height: 200px;
}

.pulse-circle {
  animation: pulse-ring 2s ease-out infinite;
}

.map-info {
  padding: var(--space-10) var(--space-8);
  display: flex;
  flex-direction: column;
  justify-content: center;
  min-width: 300px;
}

.map-info-title {
  font-size: var(--font-size-xl);
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: var(--space-3);
  line-height: 1.3;
}

.map-info-sub {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.7;
  margin-bottom: var(--space-6);
}

.operator-chips {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.operator-chip {
  font-size: var(--font-size-xs);
  font-weight: 600;
  padding: var(--space-1) var(--space-3);
  background: rgba(6, 182, 212, 0.1);
  border: 1px solid rgba(6, 182, 212, 0.2);
  border-radius: var(--radius-full);
  color: var(--color-cyan-light);
  letter-spacing: 0.02em;
}

@media (max-width: 900px) {
  .highlights-grid { grid-template-columns: repeat(2, 1fr); }
  .map-inner { grid-template-columns: 1fr; }
  .map-svg-wrap { border-right: none; border-bottom: 1px solid var(--color-border); }
  .map-info { min-width: unset; }
}

@media (max-width: 480px) {
  .highlights-grid { grid-template-columns: 1fr; }
}
</style>
