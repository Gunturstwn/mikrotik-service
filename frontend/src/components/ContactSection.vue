<script setup lang="ts">
import { ref } from 'vue'
import { company } from '@/config/company'

const form = ref({
  name: '',
  email: '',
  phone: '',
  company: '',
  message: '',
})

const isSubmitting = ref(false)
const isSubmitted = ref(false)

const handleSubmit = async () => {
  isSubmitting.value = true
  // Simulate form submission (replace with actual API call)
  await new Promise(resolve => setTimeout(resolve, 1500))
  isSubmitting.value = false
  isSubmitted.value = true
  form.value = { name: '', email: '', phone: '', company: '', message: '' }
}

const contactItems = [
  {
    icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 10c0 7-9 13-9 13s-9-6-9-13a9 9 0 0 1 18 0z"/><circle cx="12" cy="10" r="3"/></svg>`,
    label: 'Alamat Kantor',
    value: company.contact.address,
    href: '',
  },
  {
    icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M22 16.92v3a2 2 0 0 1-2.18 2 19.79 19.79 0 0 1-8.63-3.07A19.5 19.5 0 0 1 4.07 12a19.79 19.79 0 0 1-3.07-8.67A2 2 0 0 1 3 1.18h3a2 2 0 0 1 2 1.72c.127.96.361 1.903.7 2.81a2 2 0 0 1-.45 2.11L7.09 8.91A16 16 0 0 0 12 16l1.27-1.27a2 2 0 0 1 2.11-.45 12.84 12.84 0 0 0 2.81.7A2 2 0 0 1 20 17z"/></svg>`,
    label: 'Telepon',
    value: company.contact.phone,
    href: `tel:${company.contact.phone}`,
  },
  {
    icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/><polyline points="22,6 12,13 2,6"/></svg>`,
    label: 'Email',
    value: company.contact.email,
    href: `mailto:${company.contact.email}`,
  },
  {
    icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>`,
    label: 'Jam Operasional',
    value: company.contact.operationalHours,
    href: '',
  },
  {
    icon: `<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M13 10V3L4 14h7v7l9-11h-7z"/></svg>`,
    label: 'Support Darurat',
    value: company.contact.emergencySupport,
    href: '',
  },
]
</script>

<template>
  <section id="contact" class="section contact-section">
    <div class="container">
      <!-- Header -->
      <div class="section-header">
        <span class="section-label">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
            <path d="M4 4h16c1.1 0 2 .9 2 2v12c0 1.1-.9 2-2 2H4c-1.1 0-2-.9-2-2V6c0-1.1.9-2 2-2z"/>
            <polyline points="22,6 12,13 2,6"/>
          </svg>
          Hubungi Kami
        </span>
        <h2 class="section-title">{{ company.contact.title }}</h2>
        <p class="section-subtitle">{{ company.contact.subtitle }}</p>
      </div>

      <div class="contact-grid">
        <!-- Contact Info -->
        <div class="contact-info">
          <div
            v-for="(item, i) in contactItems"
            :key="i"
            class="contact-item glass-card"
          >
            <div class="contact-icon" v-html="item.icon"></div>
            <div class="contact-detail">
              <span class="contact-label">{{ item.label }}</span>
              <a v-if="item.href" :href="item.href" class="contact-value contact-link">
                {{ item.value }}
              </a>
              <span v-else class="contact-value">{{ item.value }}</span>
            </div>
          </div>

          <!-- Social Media -->
          <div class="social-row">
            <a :href="company.social.linkedin" target="_blank" rel="noopener" class="social-btn" aria-label="LinkedIn">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M16 8a6 6 0 0 1 6 6v7h-4v-7a2 2 0 0 0-2-2 2 2 0 0 0-2 2v7h-4v-7a6 6 0 0 1 6-6z"/>
                <rect x="2" y="9" width="4" height="12"/>
                <circle cx="4" cy="4" r="2"/>
              </svg>
            </a>
            <a :href="company.social.instagram" target="_blank" rel="noopener" class="social-btn" aria-label="Instagram">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="2" y="2" width="20" height="20" rx="5" ry="5"/>
                <path d="M16 11.37A4 4 0 1 1 12.63 8 4 4 0 0 1 16 11.37z"/>
                <line x1="17.5" y1="6.5" x2="17.51" y2="6.5"/>
              </svg>
            </a>
            <a :href="company.social.twitter" target="_blank" rel="noopener" class="social-btn" aria-label="Twitter">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M18.244 2.25h3.308l-7.227 8.26 8.502 11.24H16.17l-5.214-6.817L4.99 21.75H1.68l7.73-8.835L1.254 2.25H8.08l4.713 6.231zm-1.161 17.52h1.833L7.084 4.126H5.117z"/>
              </svg>
            </a>
            <a :href="company.social.youtube" target="_blank" rel="noopener" class="social-btn" aria-label="YouTube">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
                <path d="M22.54 6.42a2.78 2.78 0 0 0-1.95-1.96C18.88 4 12 4 12 4s-6.88 0-8.59.46a2.78 2.78 0 0 0-1.95 1.96A29 29 0 0 0 1 12a29 29 0 0 0 .46 5.58A2.78 2.78 0 0 0 3.41 19.6C5.12 20 12 20 12 20s6.88 0 8.59-.4a2.78 2.78 0 0 0 1.95-1.95A29 29 0 0 0 23 12a29 29 0 0 0-.46-5.58z"/>
                <polygon points="9.75 15.02 15.5 12 9.75 8.98 9.75 15.02" fill="white"/>
              </svg>
            </a>
          </div>
        </div>

        <!-- Contact Form -->
        <div class="contact-form glass-card">
          <!-- Success state -->
          <div v-if="isSubmitted" class="form-success">
            <div class="success-icon">✅</div>
            <h3>Pesan Terkirim!</h3>
            <p>Tim kami akan menghubungi Anda dalam 1x24 jam kerja.</p>
            <button class="btn btn-secondary" @click="isSubmitted = false">Kirim Pesan Lain</button>
          </div>

          <form v-else @submit.prevent="handleSubmit">
            <h3 class="form-title">Kirim Pesan</h3>

            <div class="form-row">
              <div class="form-group">
                <label class="form-label" for="contact-name">Nama Lengkap *</label>
                <input
                  id="contact-name"
                  v-model="form.name"
                  type="text"
                  class="form-input"
                  placeholder="Nama Anda"
                  required
                />
              </div>
              <div class="form-group">
                <label class="form-label" for="contact-email">Email *</label>
                <input
                  id="contact-email"
                  v-model="form.email"
                  type="email"
                  class="form-input"
                  placeholder="email@perusahaan.com"
                  required
                />
              </div>
            </div>

            <div class="form-row">
              <div class="form-group">
                <label class="form-label" for="contact-phone">No. Telepon</label>
                <input
                  id="contact-phone"
                  v-model="form.phone"
                  type="tel"
                  class="form-input"
                  placeholder="+62 8xx xxxx xxxx"
                />
              </div>
              <div class="form-group">
                <label class="form-label" for="contact-company">Perusahaan</label>
                <input
                  id="contact-company"
                  v-model="form.company"
                  type="text"
                  class="form-input"
                  placeholder="Nama perusahaan"
                />
              </div>
            </div>

            <div class="form-group">
              <label class="form-label" for="contact-message">Pesan *</label>
              <textarea
                id="contact-message"
                v-model="form.message"
                class="form-textarea"
                placeholder="Ceritakan kebutuhan layanan Anda..."
                required
              ></textarea>
            </div>

            <button
              type="submit"
              id="contact-submit"
              class="btn btn-primary form-submit"
              :disabled="isSubmitting"
            >
              <span v-if="isSubmitting">
                <svg class="spin" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
                </svg>
                Mengirim...
              </span>
              <span v-else>
                Kirim Pesan
                <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                  <line x1="22" y1="2" x2="11" y2="13"/>
                  <polygon points="22 2 15 22 11 13 2 9 22 2"/>
                </svg>
              </span>
            </button>
          </form>
        </div>
      </div>
    </div>
  </section>
</template>

<style scoped>
.contact-section {
  position: relative;
}

.section-header {
  text-align: center;
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: var(--space-12);
}

.contact-grid {
  display: grid;
  grid-template-columns: 1fr 1.5fr;
  gap: var(--space-8);
  align-items: start;
}

/* Info side */
.contact-info {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.contact-item {
  display: flex;
  align-items: flex-start;
  gap: var(--space-4);
  padding: var(--space-5) var(--space-6);
}

.contact-icon {
  flex-shrink: 0;
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(6, 182, 212, 0.1);
  border: 1px solid rgba(6, 182, 212, 0.2);
  border-radius: var(--radius-md);
  color: var(--color-cyan);
}

.contact-detail {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.contact-label {
  font-size: var(--font-size-xs);
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--color-text-muted);
}

.contact-value {
  font-size: var(--font-size-sm);
  color: var(--color-text-primary);
  line-height: 1.5;
}

.contact-link {
  color: var(--color-cyan-light);
  transition: color var(--transition-base);
}

.contact-link:hover {
  color: white;
}

/* Social */
.social-row {
  display: flex;
  gap: var(--space-3);
  padding: var(--space-2);
}

.social-btn {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
  color: var(--color-text-secondary);
  transition: all var(--transition-base);
}

.social-btn:hover {
  background: rgba(6, 182, 212, 0.12);
  border-color: rgba(6, 182, 212, 0.35);
  color: var(--color-cyan);
  transform: translateY(-2px);
}

/* Form */
.contact-form {
  padding: var(--space-8);
}

.form-title {
  font-size: var(--font-size-xl);
  font-weight: 700;
  margin-bottom: var(--space-6);
  color: var(--color-text-primary);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  margin-bottom: var(--space-4);
}

.form-submit {
  width: 100%;
  justify-content: center;
  margin-top: var(--space-2);
}

.form-submit:disabled {
  opacity: 0.7;
  cursor: not-allowed;
  transform: none;
}

/* Success State */
.form-success {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: var(--space-4);
  padding: var(--space-12) var(--space-8);
  min-height: 300px;
}

.success-icon {
  font-size: 3rem;
}

.form-success h3 {
  font-size: var(--font-size-2xl);
  font-weight: 700;
}

.form-success p {
  color: var(--color-text-secondary);
}

/* Spinner */
@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.spin { animation: spin 1s linear infinite; }

@media (max-width: 900px) {
  .contact-grid { grid-template-columns: 1fr; }
}

@media (max-width: 640px) {
  .form-row { grid-template-columns: 1fr; }
}
</style>
