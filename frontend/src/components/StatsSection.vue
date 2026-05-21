<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { company } from '@/config/company'

const displayValues = ref(company.stats.map(() => 0))
let animated = false

const animateCounters = () => {
  if (animated) return
  animated = true

  company.stats.forEach((stat, i) => {
    const target = stat.value
    const isFloat = 'isFloat' in stat && stat.isFloat
    const duration = 2000
    const steps = 60
    const increment = target / steps
    let current = 0
    let step = 0

    const timer = setInterval(() => {
      step++
      current = Math.min(increment * step, target)
      displayValues.value[i] = isFloat ? parseFloat(current.toFixed(1)) : Math.floor(current)
      if (step >= steps) clearInterval(timer)
    }, duration / steps)
  })
}

onMounted(() => {
  const el = document.querySelector('.stats-section')
  const observer = new IntersectionObserver(
    (entries) => { if (entries[0].isIntersecting) animateCounters() },
    { threshold: 0.3 }
  )
  if (el) observer.observe(el)
})
</script>

<template>
  <section class="stats-section">
    <div class="container">
      <div class="stats-grid">
        <div
          v-for="(stat, i) in company.stats"
          :key="i"
          class="stat-card"
        >
          <div class="stat-glow"></div>
          <div class="stat-value">
            <span class="stat-number">{{ displayValues[i] }}</span>
            <span class="stat-suffix">{{ stat.suffix }}</span>
          </div>
          <p class="stat-label">{{ stat.label }}</p>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.stats-section {
  padding: var(--space-20) 0;
  position: relative;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-6);
}

.stat-card {
  position: relative;
  text-align: center;
  padding: var(--space-10) var(--space-6);
  background: var(--color-bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-xl);
  overflow: hidden;
  transition: border-color var(--transition-base), box-shadow var(--transition-base), transform var(--transition-slow);
}

.stat-card:hover {
  border-color: var(--color-border-hover);
  box-shadow: var(--shadow-card-hover);
  transform: translateY(-4px);
}

.stat-glow {
  position: absolute;
  inset: 0;
  background: radial-gradient(ellipse at 50% 0%, rgba(6, 182, 212, 0.08) 0%, transparent 60%);
  pointer-events: none;
}

.stat-value {
  display: flex;
  align-items: baseline;
  justify-content: center;
  gap: 2px;
  margin-bottom: var(--space-3);
}

.stat-number {
  font-size: clamp(2.5rem, 4vw, 3.5rem);
  font-weight: 900;
  letter-spacing: -0.03em;
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  line-height: 1;
}

.stat-suffix {
  font-size: var(--font-size-2xl);
  font-weight: 700;
  background: var(--gradient-text);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.stat-label {
  font-size: var(--font-size-sm);
  font-weight: 500;
  color: var(--color-text-secondary);
  letter-spacing: 0.02em;
  text-transform: uppercase;
}

@media (max-width: 900px) {
  .stats-grid { grid-template-columns: repeat(2, 1fr); }
}

@media (max-width: 480px) {
  .stats-grid { grid-template-columns: 1fr 1fr; gap: var(--space-4); }
  .stat-card { padding: var(--space-8) var(--space-4); }
}
</style>
