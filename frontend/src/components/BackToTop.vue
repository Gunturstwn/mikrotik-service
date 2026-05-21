<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'

const visible = ref(false)

const handleScroll = () => {
  visible.value = window.scrollY > 600
}

const scrollTop = () => {
  window.scrollTo({ top: 0, behavior: 'smooth' })
}

onMounted(() => window.addEventListener('scroll', handleScroll))
onUnmounted(() => window.removeEventListener('scroll', handleScroll))
</script>

<template>
  <Transition name="fade-up">
    <button
      v-show="visible"
      @click="scrollTop"
      id="back-to-top-btn"
      class="back-to-top"
      aria-label="Kembali ke atas"
    >
      <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
        <path d="M18 15l-6-6-6 6"/>
      </svg>
    </button>
  </Transition>
</template>

<style scoped>
.back-to-top {
  position: fixed;
  bottom: var(--space-8);
  right: var(--space-8);
  z-index: 99;
  width: 48px;
  height: 48px;
  background: var(--gradient-primary);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  box-shadow: 0 4px 20px rgba(6, 182, 212, 0.4);
  transition: transform var(--transition-base), box-shadow var(--transition-base);
  cursor: pointer;
  border: none;
}

.back-to-top:hover {
  transform: translateY(-3px);
  box-shadow: 0 8px 32px rgba(6, 182, 212, 0.6);
}

.fade-up-enter-active,
.fade-up-leave-active {
  transition: opacity 0.3s ease, transform 0.3s ease;
}

.fade-up-enter-from,
.fade-up-leave-to {
  opacity: 0;
  transform: translateY(12px);
}

@media (max-width: 640px) {
  .back-to-top {
    bottom: var(--space-5);
    right: var(--space-5);
    width: 42px;
    height: 42px;
  }
}
</style>
