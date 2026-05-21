import { fileURLToPath, URL } from 'node:url'
import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  server: {
    proxy: {
      // Proxy API requests ke backend
      '/api': {
        target: 'http://localhost:5150',
        changeOrigin: true,
      },
      // Proxy MinIO image requests agar foto bisa diakses dari frontend
      '/mikrotik-images': {
        target: 'http://localhost:9000',
        changeOrigin: true,
      },
    }
  }
})
