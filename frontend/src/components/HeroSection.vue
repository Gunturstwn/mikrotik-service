<script setup lang="ts">
import { company } from '@/config/company'
</script>

<template>
  <section id="top" class="hero">
    <!-- Animated background grid -->
    <div class="hero-grid" aria-hidden="true"></div>

    <!-- Glowing orbs -->
    <div class="orb orb-1" aria-hidden="true"></div>
    <div class="orb orb-2" aria-hidden="true"></div>
    <div class="orb orb-3" aria-hidden="true"></div>

    <div class="container hero-content">
      <!-- Badge -->
      <div class="hero-badge animate-fade-up" style="animation-delay: 0.1s">
        <span class="badge-dot"></span>
        Berdiri Sejak {{ company.foundedYear }} · Terpercaya Nasional
      </div>

      <!-- Headline -->
      <h1 class="hero-title animate-fade-up" style="animation-delay: 0.2s">
        <span>Membangun Indonesia</span>
        <br/>
        <span class="gradient-text">dengan Teknologi</span>
      </h1>

      <!-- Subheadline -->
      <p class="hero-subtitle animate-fade-up" style="animation-delay: 0.35s">
        {{ company.subTagline }}
      </p>

      <!-- CTA Buttons -->
      <div class="hero-actions animate-fade-up" style="animation-delay: 0.5s">
        <a
          :href="company.ctaPrimary.href"
          id="hero-cta-primary"
          class="btn btn-primary"
          @click.prevent="$el.closest('body').querySelector(company.ctaPrimary.href)?.scrollIntoView({ behavior: 'smooth' })"
        >
          <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M13 10V3L4 14h7v7l9-11h-7z"/>
          </svg>
          {{ company.ctaPrimary.label }}
        </a>
        <a
          :href="company.ctaSecondary.href"
          id="hero-cta-secondary"
          class="btn btn-secondary"
        >
          {{ company.ctaSecondary.label }}
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M5 12h14M12 5l7 7-7 7"/>
          </svg>
        </a>
      </div>

      <!-- Floating Feature Pills -->
      <div class="hero-pills animate-fade-up" style="animation-delay: 0.65s">
        <span class="pill">✅ 99.9% Uptime SLA</span>
        <span class="pill">🔒 Keamanan Enterprise</span>
        <span class="pill">⚡ NOC 24/7</span>
        <span class="pill">🌐 Jangkauan Nasional</span>
      </div>
    </div>

    <!-- Scroll Indicator -->
    <div class="scroll-indicator">
      <div class="scroll-line"></div>
      <span>Scroll</span>
    </div>
  </section>
</template>

<style scoped>
.hero {
  position: relative;
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  background: var(--gradient-hero);
  padding-top: var(--navbar-height);
}

/* Animated grid background */
.hero-grid {
  position: absolute;
  inset: 0;
  background-image:
    linear-gradient(rgba(6, 182, 212, 0.04) 1px, transparent 1px),
    linear-gradient(90deg, rgba(6, 182, 212, 0.04) 1px, transparent 1px);
  background-size: 60px 60px;
  mask-image: radial-gradient(ellipse 80% 80% at 50% 50%, black 40%, transparent 100%);
}

/* Glowing orbs */
.orb {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  pointer-events: none;
}

.orb-1 {
  width: 600px;
  height: 600px;
  background: radial-gradient(circle, rgba(6, 182, 212, 0.18) 0%, transparent 70%);
  top: -200px;
  right: -100px;
  animation: float 8s ease-in-out infinite;
}

.orb-2 {
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.15) 0%, transparent 70%);
  bottom: -100px;
  left: -100px;
  animation: float 10s ease-in-out infinite reverse;
}

.orb-3 {
  width: 300px;
  height: 300px;
  background: radial-gradient(circle, rgba(139, 92, 246, 0.12) 0%, transparent 70%);
  top: 40%;
  left: 20%;
  animation: float 7s ease-in-out infinite 2s;
}

/* Content */
.hero-content {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;
  padding-top: var(--space-16);
  padding-bottom: var(--space-20);
}

.hero-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-cyan-light);
  padding: var(--space-2) var(--space-5);
  background: rgba(6, 182, 212, 0.08);
  border: 1px solid rgba(6, 182, 212, 0.2);
  border-radius: var(--radius-full);
  margin-bottom: var(--space-8);
  letter-spacing: 0.02em;
}

.badge-dot {
  width: 8px;
  height: 8px;
  background: var(--color-cyan);
  border-radius: 50%;
  position: relative;
}

.badge-dot::after {
  content: '';
  position: absolute;
  inset: 0;
  background: var(--color-cyan);
  border-radius: 50%;
  animation: pulse-ring 1.5s ease-out infinite;
}

.hero-title {
  font-size: clamp(2.5rem, 7vw, 5.5rem);
  font-weight: 900;
  line-height: 1.05;
  letter-spacing: -0.03em;
  margin-bottom: var(--space-8);
  max-width: 900px;
}

.hero-subtitle {
  font-size: clamp(var(--font-size-base), 2vw, var(--font-size-xl));
  color: var(--color-text-secondary);
  max-width: 620px;
  line-height: 1.75;
  margin-bottom: var(--space-10);
}

.hero-actions {
  display: flex;
  gap: var(--space-4);
  flex-wrap: wrap;
  justify-content: center;
  margin-bottom: var(--space-12);
}

.hero-actions .btn {
  min-width: 180px;
}

/* Pills */
.hero-pills {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
  justify-content: center;
}

.pill {
  font-size: var(--font-size-sm);
  font-weight: 500;
  padding: var(--space-2) var(--space-4);
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-full);
  color: var(--color-text-secondary);
  transition: all var(--transition-base);
}

.pill:hover {
  background: rgba(6, 182, 212, 0.08);
  border-color: rgba(6, 182, 212, 0.3);
  color: var(--color-text-primary);
}

/* Scroll Indicator */
.scroll-indicator {
  position: absolute;
  bottom: var(--space-8);
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-text-muted);
  font-size: var(--font-size-xs);
  letter-spacing: 0.1em;
  text-transform: uppercase;
  animation: fadeIn 1s ease 1s both;
}

.scroll-line {
  width: 1px;
  height: 40px;
  background: linear-gradient(180deg, var(--color-cyan), transparent);
  animation: shimmer 2s ease-in-out infinite;
}

@media (max-width: 768px) {
  .hero-actions { flex-direction: column; align-items: stretch; }
  .hero-actions .btn { min-width: unset; }
  .hero-pills { gap: var(--space-2); }
  .scroll-indicator { display: none; }
}
</style>
