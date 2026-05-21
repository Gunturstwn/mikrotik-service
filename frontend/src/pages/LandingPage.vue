<script setup lang="ts">
import { onMounted } from 'vue'
import AppNavbar from '@/components/AppNavbar.vue'
import HeroSection from '@/components/HeroSection.vue'
import StatsSection from '@/components/StatsSection.vue'
import AboutSection from '@/components/AboutSection.vue'
import ServicesSection from '@/components/ServicesSection.vue'
import CoverageSection from '@/components/CoverageSection.vue'
import PartnersSection from '@/components/PartnersSection.vue'
import ContactSection from '@/components/ContactSection.vue'
import AppFooter from '@/components/AppFooter.vue'
import BackToTop from '@/components/BackToTop.vue'

onMounted(() => {
  const observer = new IntersectionObserver(
    (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) entry.target.classList.add('visible')
      })
    },
    { threshold: 0.08, rootMargin: '0px 0px -60px 0px' }
  )
  document.querySelectorAll('.reveal-section').forEach((el) => observer.observe(el))
})
</script>

<template>
  <div id="app-root">
    <AppNavbar />
    <main>
      <HeroSection />
      <div class="reveal-section"><StatsSection /></div>
      <div class="divider"></div>
      <div class="reveal-section"><AboutSection /></div>
      <div class="divider"></div>
      <div class="reveal-section"><ServicesSection /></div>
      <div class="divider"></div>
      <div class="reveal-section"><CoverageSection /></div>
      <div class="divider"></div>
      <div class="reveal-section"><PartnersSection /></div>
      <div class="divider"></div>
      <div class="reveal-section"><ContactSection /></div>
    </main>
    <AppFooter />
    <BackToTop />
  </div>
</template>

<style>
.reveal-section {
  opacity: 0;
  transform: translateY(28px);
  transition: opacity 0.7s ease, transform 0.7s ease;
}
.reveal-section.visible {
  opacity: 1;
  transform: translateY(0);
}
</style>
