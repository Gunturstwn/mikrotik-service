<script setup lang="ts">
import { company } from '@/config/company'
</script>

<template>
  <section id="about" class="section about-section">
    <div class="container">
      <div class="about-grid">
        <!-- Left: Text Content -->
        <div class="about-text">
          <span class="section-label">
            <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <circle cx="12" cy="12" r="10"/><path d="M12 8v4l3 3"/>
            </svg>
            Tentang Kami
          </span>
          <h2 class="section-title">{{ company.about.title }}</h2>

          <div class="about-paragraphs">
            <p v-for="(para, i) in company.about.paragraphs" :key="i" class="about-para">
              {{ para }}
            </p>
          </div>

          <a href="#contact" class="btn btn-primary about-cta"
            @click.prevent="document.querySelector('#contact')?.scrollIntoView({ behavior: 'smooth' })">
            Pelajari Lebih Lanjut
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
              <path d="M5 12h14M12 5l7 7-7 7"/>
            </svg>
          </a>
        </div>

        <!-- Right: Vision & Mission -->
        <div class="about-cards">
          <!-- Vision Card -->
          <div class="vm-card glass-card">
            <div class="vm-icon vm-icon-vision">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/>
                <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
                <line x1="2" y1="12" x2="22" y2="12"/>
              </svg>
            </div>
            <h3 class="vm-title">Visi Kami</h3>
            <p class="vm-text">{{ company.about.vision }}</p>
          </div>

          <!-- Mission Card -->
          <div class="vm-card glass-card">
            <div class="vm-icon vm-icon-mission">
              <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="9 11 12 14 22 4"/>
                <path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/>
              </svg>
            </div>
            <h3 class="vm-title">Misi Kami</h3>
            <ul class="vm-list">
              <li v-for="(item, i) in company.about.mission" :key="i">
                <span class="vm-bullet">
                  <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
                    <polyline points="20 6 9 17 4 12"/>
                  </svg>
                </span>
                {{ item }}
              </li>
            </ul>
          </div>

          <!-- Decorative element -->
          <div class="about-deco" aria-hidden="true">
            <div class="deco-ring deco-ring-1"></div>
            <div class="deco-ring deco-ring-2"></div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.about-section {
  position: relative;
  background: linear-gradient(180deg, transparent, rgba(6, 182, 212, 0.03) 50%, transparent);
}

.about-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-16);
  align-items: center;
}

/* Text Side */
.about-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}

.about-paragraphs {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
  margin-bottom: var(--space-8);
}

.about-para {
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  line-height: 1.8;
}

.about-cta {
  align-self: flex-start;
}

/* Cards Side */
.about-cards {
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
  position: relative;
}

.vm-card {
  padding: var(--space-8);
}

.vm-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 52px;
  height: 52px;
  border-radius: var(--radius-lg);
  margin-bottom: var(--space-4);
}

.vm-icon-vision {
  background: rgba(6, 182, 212, 0.12);
  color: var(--color-cyan);
  border: 1px solid rgba(6, 182, 212, 0.2);
}

.vm-icon-mission {
  background: rgba(59, 130, 246, 0.12);
  color: var(--color-blue);
  border: 1px solid rgba(59, 130, 246, 0.2);
}

.vm-title {
  font-size: var(--font-size-lg);
  font-weight: 700;
  color: var(--color-text-primary);
  margin-bottom: var(--space-3);
}

.vm-text {
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.75;
}

.vm-list {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.vm-list li {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  font-size: var(--font-size-sm);
  color: var(--color-text-secondary);
  line-height: 1.6;
}

.vm-bullet {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: rgba(6, 182, 212, 0.12);
  border-radius: 50%;
  flex-shrink: 0;
  margin-top: 2px;
  color: var(--color-cyan);
}

/* Decorative rings */
.about-deco {
  position: absolute;
  right: -60px;
  bottom: -60px;
  pointer-events: none;
  z-index: -1;
}

.deco-ring {
  position: absolute;
  border-radius: 50%;
  border: 1px solid rgba(6, 182, 212, 0.1);
}

.deco-ring-1 {
  width: 200px;
  height: 200px;
  top: -100px;
  left: -100px;
}

.deco-ring-2 {
  width: 300px;
  height: 300px;
  top: -150px;
  left: -150px;
  border-color: rgba(59, 130, 246, 0.07);
}

@media (max-width: 1024px) {
  .about-grid { gap: var(--space-10); }
}

@media (max-width: 768px) {
  .about-grid { grid-template-columns: 1fr; }
  .about-cards { margin-top: var(--space-4); }
  .about-deco { display: none; }
}
</style>
