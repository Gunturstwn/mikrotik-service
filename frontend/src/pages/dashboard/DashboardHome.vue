<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { getMyProfile, type UserProfileResponse } from '@/api/user'
import { listMikrotikClients, getMikrotikInterfaces, getMikrotikResource, type MikrotikClientResponse, type MikrotikInterfaceResponse, type MikrotikResourceResponse } from '@/api/mikrotik'
import { queryInterfaceMetrics, type InterfaceMetricsDataPoint } from '@/api/metrics'
import {
  getCachedDevices, setCachedDevices,
  getCachedIfaceMap, setCachedIfaceMap,
  getCachedResourceMap, setCachedResourceMap,
  getCachedMetricsMap, setCachedMetricsMap,
} from '@/cache/dashboardCache'
import { useRouter } from 'vue-router'

const router = useRouter()

const user = ref<UserProfileResponse | null>(null)
const devices = ref<MikrotikClientResponse[]>([])
const ifaceMap = ref<Record<string, MikrotikInterfaceResponse[]>>({})
const resourceMap = ref<Record<string, MikrotikResourceResponse>>({})
const metricsMap = ref<Record<string, InterfaceMetricsDataPoint[]>>({})
const isLoading = ref(true)

function formatBits(val: number): string {
  if (val >= 1_000_000_000) return (val / 1_000_000_000).toFixed(1) + ' Gbps'
  if (val >= 1_000_000) return (val / 1_000_000).toFixed(1) + ' Mbps'
  if (val >= 1_000) return (val / 1_000).toFixed(1) + ' Kbps'
  return val.toFixed(0) + ' bps'
}

function formatUptime(raw: string): string {
  // RouterOS format: 1w2d14h32m18s or 2d14h32m or 32m18s or just 18s
  const w = raw.match(/(\d+)w/)?.[1]
  const d = raw.match(/(\d+)d/)?.[1]
  const h = raw.match(/(\d+)h/)?.[1]
  const m = raw.match(/(\d+)m/)?.[1]
  const s = raw.match(/(\d+)s/)?.[1]
  const parts: string[] = []
  if (w) parts.push(`${w}w`)
  if (d) parts.push(`${d}d`)
  if (h) parts.push(`${h}h`)
  if (m) parts.push(`${m}m`)
  if (s && !w && !d && !h) parts.push(`${s}s`)
  return parts.join(' ') || raw
}

function makeSparklinePath(data: number[], w: number, h: number): string {
  if (data.length < 2) return ''
  const max = Math.max(...data, 1)
  const stepX = w / (data.length - 1)
  return data.map((v, i) => {
    const x = i * stepX
    const y = h - (v / max) * h
    return `${i === 0 ? 'M' : 'L'}${x.toFixed(1)},${y.toFixed(1)}`
  }).join(' ')
}

function makeSparklineArea(data: number[], w: number, h: number): string {
  if (data.length < 2) return ''
  const max = Math.max(...data, 1)
  const stepX = w / (data.length - 1)
  const pts = data.map((v, i) => {
    const x = i * stepX
    const y = h - (v / max) * h
    return `${x.toFixed(1)},${y.toFixed(1)}`
  })
  return `M0,${h} L${pts.join(' L')} L${((data.length - 1) * stepX).toFixed(1)},${h} Z`
}

function formatBytes(val: number): string {
  if (val >= 1_073_741_824) return (val / 1_073_741_824).toFixed(1) + ' GB'
  if (val >= 1_048_576) return (val / 1_048_576).toFixed(1) + ' MB'
  if (val >= 1_024) return (val / 1_024).toFixed(1) + ' KB'
  return val.toFixed(0) + ' B'
}

/** Find the "primary" interface: first running one, or the first non-loopback */
function primaryInterface(ifaces: MikrotikInterfaceResponse[]): MikrotikInterfaceResponse | null {
  const running = ifaces.find(i => i.running && !i.disabled && !i.name.startsWith('lo'))
  return running ?? (ifaces.find(i => !i.name.startsWith('lo')) ?? ifaces[0] ?? null)
}

type DeviceWithBW = MikrotikClientResponse & {
  mainIface: MikrotikInterfaceResponse | null
}

const devicesWithBW = computed<DeviceWithBW[]>(() => {
  return devices.value.map(d => ({
    ...d,
    mainIface: primaryInterface(ifaceMap.value[d.id] ?? []),
  }))
})

const totalRX = computed(() => devicesWithBW.value.reduce((s, d) => s + (d.mainIface?.rx_byte ?? 0), 0))
const totalTX = computed(() => devicesWithBW.value.reduce((s, d) => s + (d.mainIface?.tx_byte ?? 0), 0))

const avgCPU = computed(() => {
  const vals = devices.value.map(d => resourceMap.value[d.id]).filter(Boolean) as MikrotikResourceResponse[]
  if (vals.length === 0) return 0
  return vals.reduce((s, r) => s + r.cpu_load, 0) / vals.length
})

const totalRAM = computed(() => devices.value.reduce((s, d) => {
  const r = resourceMap.value[d.id]
  return s + (r ? r.total_memory : 0)
}, 0))

const usedRAM = computed(() => devices.value.reduce((s, d) => {
  const r = resourceMap.value[d.id]
  return s + (r ? r.total_memory - r.free_memory : 0)
}, 0))

const totalDisk = computed(() => devices.value.reduce((s, d) => {
  const r = resourceMap.value[d.id]
  return s + (r ? r.total_hdd_space : 0)
}, 0))

const usedDisk = computed(() => devices.value.reduce((s, d) => {
  const r = resourceMap.value[d.id]
  return s + (r ? r.total_hdd_space - r.free_hdd_space : 0)
}, 0))

onMounted(async () => {
  try {
    // 1. Fetch user profile first to build a per-user cache key
    const u = await getMyProfile()
    user.value = u
    const CACHE_KEY = `dashboard-${u.id}`

    // 2. Load cached data immediately (stale-while-revalidate)
    const cachedDevices = getCachedDevices(CACHE_KEY)
    if (cachedDevices) {
      devices.value = cachedDevices
      const cachedIfaceMap = getCachedIfaceMap(CACHE_KEY)
      const cachedResMap = getCachedResourceMap(CACHE_KEY)
      const cachedMetricsMap = getCachedMetricsMap(CACHE_KEY)
      if (cachedIfaceMap) ifaceMap.value = cachedIfaceMap
      if (cachedResMap) resourceMap.value = cachedResMap
      if (cachedMetricsMap) metricsMap.value = cachedMetricsMap
      isLoading.value = false
    }

    // 3. Always fetch fresh data in background
    const d = await listMikrotikClients()
    devices.value = d

    // Fetch interfaces + resources in parallel
    const ifacePromise = Promise.allSettled(d.map(dev => getMikrotikInterfaces(dev.id)))
    const resPromise = Promise.allSettled(d.map(dev => getMikrotikResource(dev.id)))

    // Start metrics fetch as soon as interfaces arrive (chain, not await)
    const metricsPromise = ifacePromise.then(ifaceResults => {
      const tmpIfaceMap: Record<string, MikrotikInterfaceResponse[]> = {}
      for (let i = 0; i < d.length; i++) {
        if (ifaceResults[i].status === 'fulfilled') {
          tmpIfaceMap[d[i].id] = ifaceResults[i].value
        }
      }

      const endDate = new Date()
      const startDate = new Date(Date.now() - 24 * 60 * 60 * 1000)
      const fmtDate = (dt: Date) => {
        const p = (n: number) => String(n).padStart(2, '0')
        return `${dt.getFullYear()}-${p(dt.getMonth()+1)}-${p(dt.getDate())}T${p(dt.getHours())}:${p(dt.getMinutes())}:${p(dt.getSeconds())}`
      }

      return Promise.allSettled(
        d.map(dev => {
          const primaryName = primaryInterface(tmpIfaceMap[dev.id] ?? [])?.name
          if (!primaryName) return 'skipped'
          return queryInterfaceMetrics(dev.id, {
            interface_name: primaryName,
            aggregation: 'hourly',
            start_date: fmtDate(startDate),
            end_date: fmtDate(endDate),
            page_size: 50,
          })
        })
      )
    })

    const [ifaceResults, resResults, metricsResults] = await Promise.all([
      ifacePromise,
      resPromise,
      metricsPromise,
    ])

    // Build fresh maps
    const newIfaceMap: Record<string, MikrotikInterfaceResponse[]> = {}
    const newResMap: Record<string, MikrotikResourceResponse> = {}
    const newMetricsMap: Record<string, InterfaceMetricsDataPoint[]> = {}

    for (let i = 0; i < d.length; i++) {
      if (ifaceResults[i].status === 'fulfilled') {
        newIfaceMap[d[i].id] = ifaceResults[i].value
        ifaceMap.value[d[i].id] = ifaceResults[i].value
      }
      if (resResults[i].status === 'fulfilled') {
        newResMap[d[i].id] = resResults[i].value
        resourceMap.value[d[i].id] = resResults[i].value
      }
    }
    for (let i = 0; i < d.length; i++) {
      const r = metricsResults[i]
      if (r.status === 'fulfilled' && typeof r.value === 'object' && 'items' in r.value) {
        newMetricsMap[d[i].id] = r.value.items
        metricsMap.value[d[i].id] = r.value.items
      }
    }

    // 4. Update cache with fresh data
    setCachedDevices(CACHE_KEY, d)
    setCachedIfaceMap(CACHE_KEY, newIfaceMap)
    setCachedResourceMap(CACHE_KEY, newResMap)
    setCachedMetricsMap(CACHE_KEY, newMetricsMap)
  } catch { /* handled by layout auth guard */ }
  isLoading.value = false
})
</script>

<template>
  <div class="home">
    <div v-if="isLoading" class="loading">
      <svg class="spin" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 12a9 9 0 1 1-6.219-8.56"/>
      </svg>
    </div>
    <template v-else>
      <h1 class="page-title">Selamat Datang, {{ user?.name?.split(' ')[0] ?? 'User' }}</h1>
      <p class="page-sub">Ringkasan akun dan perangkat Anda.</p>

      <!-- Stats Cards -->
      <div class="stats-grid">
        <div class="stat-card glass-card">
          <div class="stat-icon blue">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="14" width="20" height="7" rx="2"/>
              <path d="M6 18h.01M10 18h.01"/>
              <path d="M12 3v4M8 7h8M7 7l5-4 5 4"/>
            </svg>
          </div>
          <div>
            <span class="stat-number">{{ devices.length }}</span>
            <span class="stat-label">Perangkat MikroTik</span>
          </div>
        </div>
        <div class="stat-card glass-card">
          <div class="stat-icon cyan">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/>
            </svg>
          </div>
          <div>
            <span class="stat-number">{{ user?.is_verified ? 'Aktif' : 'Pending' }}</span>
            <span class="stat-label">Status Akun</span>
          </div>
        </div>
        <div class="stat-card glass-card">
          <div class="stat-icon violet">
            <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/>
              <circle cx="12" cy="7" r="4"/>
            </svg>
          </div>
          <div>
            <span class="stat-number">{{ user?.roles?.join(', ') || 'User' }}</span>
            <span class="stat-label">Role</span>
          </div>
        </div>
      </div>

      <!-- Aggregate Resource Summary -->
      <div v-if="devices.length > 0" class="resource-summary-section">
        <h2 class="section-title" style="margin-bottom: var(--space-4);">Resource Summary</h2>
        <div class="res-summary-grid">
          <div class="res-summary-card glass-card">
            <div class="res-summary-icon cpu">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/>
                <path d="M9 1v3M15 1v3M9 20v3M15 20v3M1 9h3M20 9h3M1 15h3M20 15h3"/>
              </svg>
            </div>
            <div class="res-summary-body">
              <span class="res-summary-label">CPU Rata-rata</span>
              <div class="res-bar-track">
                <div class="res-bar-fill cpu-fill" :style="{ width: avgCPU + '%' }"></div>
              </div>
              <span class="res-summary-value">{{ avgCPU.toFixed(1) }}%</span>
            </div>
          </div>
          <div class="res-summary-card glass-card">
            <div class="res-summary-icon mem">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <rect x="4" y="4" width="16" height="16" rx="2"/>
                <line x1="9" y1="4" x2="9" y2="20"/><line x1="15" y1="4" x2="15" y2="20"/>
                <line x1="4" y1="9" x2="20" y2="9"/><line x1="4" y1="15" x2="20" y2="15"/>
              </svg>
            </div>
            <div class="res-summary-body">
              <span class="res-summary-label">RAM Terpakai</span>
              <div class="res-bar-track">
                <div class="res-bar-fill mem-fill" :style="{ width: (totalRAM > 0 ? (usedRAM / totalRAM * 100) : 0) + '%' }"></div>
              </div>
              <span class="res-summary-value">{{ formatBytes(usedRAM) }} / {{ formatBytes(totalRAM) }}</span>
            </div>
          </div>
          <div class="res-summary-card glass-card">
            <div class="res-summary-icon disk">
              <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <ellipse cx="12" cy="5" rx="9" ry="3"/>
                <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"/>
                <path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/>
              </svg>
            </div>
            <div class="res-summary-body">
              <span class="res-summary-label">Disk Terpakai</span>
              <div class="res-bar-track">
                <div class="res-bar-fill disk-fill" :style="{ width: (totalDisk > 0 ? (usedDisk / totalDisk * 100) : 0) + '%' }"></div>
              </div>
              <span class="res-summary-value">{{ formatBytes(usedDisk) }} / {{ formatBytes(totalDisk) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- Bandwidth + Resource per Device -->
      <div v-if="devices.length > 0" class="bandwidth-section">
        <div class="section-header">
          <h2 class="section-title">Perangkat &amp; Kinerja</h2>
          <div class="total-bw">
            <span class="total-rx">RX: {{ formatBits(totalRX) }}</span>
            <span class="total-tx">TX: {{ formatBits(totalTX) }}</span>
          </div>
        </div>

        <div class="bw-grid">
          <div
            v-for="d in devicesWithBW"
            :key="d.id"
            class="bw-card glass-card"
            @click="router.push('/dashboard/mikrotik')"
          >
            <div class="bw-header">
              <div class="bw-name">{{ d.name_device }}</div>
              <span class="bw-host">{{ d.host }}</span>
            </div>

            <!-- Uptime -->
            <div v-if="resourceMap[d.id]" class="bw-uptime">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/>
              </svg>
              {{ formatUptime(resourceMap[d.id].uptime) }}
            </div>

            <!-- Sparkline 24h -->
            <div v-if="metricsMap[d.id] && metricsMap[d.id].length >= 2" class="sparkline-wrap">
              <svg width="100%" height="44" viewBox="0 0 180 44" preserveAspectRatio="none" class="sparkline-svg">
                <path :d="makeSparklineArea(metricsMap[d.id].map(p => p.rx_byte), 180, 40)" class="spark-area spark-area-rx" />
                <path :d="makeSparklinePath(metricsMap[d.id].map(p => p.rx_byte), 180, 40)" class="spark-line spark-line-rx" />
                <path :d="makeSparklineArea(metricsMap[d.id].map(p => p.tx_byte), 180, 40)" class="spark-area spark-area-tx" />
                <path :d="makeSparklinePath(metricsMap[d.id].map(p => p.tx_byte), 180, 40)" class="spark-line spark-line-tx" />
              </svg>
              <div class="sparkline-label">
                <span class="spark-rx">RX 24j</span>
                <span class="spark-tx">TX 24j</span>
              </div>
            </div>

            <!-- Bandwidth -->
            <div v-if="d.mainIface" class="bw-metrics">
              <div class="bw-row">
                <div class="bw-metric down">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="8 17 12 21 16 17"/><polyline points="12 5 12 21"/>
                    <line x1="4" y1="21" x2="20" y2="21"/>
                  </svg>
                  <div>
                    <span class="bw-label">Download</span>
                    <span class="bw-value">{{ formatBytes(d.mainIface.rx_byte ?? 0) }}</span>
                  </div>
                </div>
                <div class="bw-metric up">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <polyline points="17 8 12 3 7 8"/><polyline points="12 21 12 3"/>
                    <line x1="4" y1="3" x2="20" y2="3"/>
                  </svg>
                  <div>
                    <span class="bw-label">Upload</span>
                    <span class="bw-value">{{ formatBytes(d.mainIface.tx_byte ?? 0) }}</span>
                  </div>
                </div>
              </div>
              <div class="bw-interface">
                <span :class="['bw-status-dot', d.mainIface.running && !d.mainIface.disabled ? 'online' : 'offline']" />
                {{ d.mainIface.name }}
              </div>
            </div>
            <div v-else class="bw-empty">
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/>
                <line x1="12" y1="16" x2="12.01" y2="16"/>
              </svg>
              <span>Belum ada data</span>
            </div>

            <!-- Tooltip Detail -->
            <div class="bw-tooltip">
              <div class="bw-tooltip-arrow"></div>
              <div class="bw-tooltip-body">
                <div class="tt-row">
                  <span class="tt-label">Host</span>
                  <span class="tt-val">{{ d.host }}</span>
                </div>
                <div v-if="resourceMap[d.id]" class="tt-row">
                  <span class="tt-label">Uptime</span>
                  <span class="tt-val">{{ resourceMap[d.id].uptime }}</span>
                </div>
                <div class="tt-divider"></div>
                <div class="tt-section-title">Bandwidth</div>
                <template v-if="d.mainIface">
                  <div class="tt-row">
                    <span class="tt-label">Interface</span>
                    <span class="tt-val mono">{{ d.mainIface.name }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label tt-rx">Download</span>
                    <span class="tt-val mono">{{ formatBytes(d.mainIface.rx_byte ?? 0) }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label tt-tx">Upload</span>
                    <span class="tt-val mono">{{ formatBytes(d.mainIface.tx_byte ?? 0) }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label">RX Packet</span>
                    <span class="tt-val mono">{{ (d.mainIface.rx_packet ?? 0).toLocaleString() }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label">TX Packet</span>
                    <span class="tt-val mono">{{ (d.mainIface.tx_packet ?? 0).toLocaleString() }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label">RX Error</span>
                    <span class="tt-val mono" :class="{ 'tt-warn': (d.mainIface.rx_error ?? 0) > 0 }">{{ (d.mainIface.rx_error ?? 0).toLocaleString() }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label">TX Error</span>
                    <span class="tt-val mono" :class="{ 'tt-warn': (d.mainIface.tx_error ?? 0) > 0 }">{{ (d.mainIface.tx_error ?? 0).toLocaleString() }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label">Status</span>
                    <span class="tt-val" :class="d.mainIface.running && !d.mainIface.disabled ? 'tt-online' : 'tt-offline'">
                      {{ d.mainIface.running && !d.mainIface.disabled ? 'Online' : 'Offline' }}
                    </span>
                  </div>
                </template>
                <div v-else class="tt-na">Tidak ada data interface</div>
                <div class="tt-divider"></div>
                <div class="tt-section-title">Resources</div>
                <template v-if="resourceMap[d.id]">
                  <div class="tt-row">
                    <span class="tt-label tt-cpu">CPU</span>
                    <span class="tt-val mono">{{ resourceMap[d.id].cpu_load }}%</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label tt-mem">RAM</span>
                    <span class="tt-val mono">{{ formatBytes(resourceMap[d.id].total_memory - resourceMap[d.id].free_memory) }} / {{ formatBytes(resourceMap[d.id].total_memory) }}</span>
                  </div>
                  <div class="tt-row">
                    <span class="tt-label tt-disk">Disk</span>
                    <span class="tt-val mono">{{ formatBytes(resourceMap[d.id].total_hdd_space - resourceMap[d.id].free_hdd_space) }} / {{ formatBytes(resourceMap[d.id].total_hdd_space) }}</span>
                  </div>
                </template>
                <div v-else class="tt-na">Resource tidak tersedia</div>
                <div class="tt-divider"></div>
                <div class="tt-click-hint">Klik untuk detail lengkap</div>
              </div>
            </div>
            <template v-if="resourceMap[d.id]">
              <div class="res-divider"></div>
              <div class="res-bars">
                <div class="res-bar-item">
                  <div class="res-bar-head">
                    <span class="res-bar-label">CPU</span>
                    <span class="res-bar-pct">{{ resourceMap[d.id].cpu_load }}%</span>
                  </div>
                  <div class="res-bar-track">
                    <div class="res-bar-fill cpu-fill" :style="{ width: resourceMap[d.id].cpu_load + '%' }"></div>
                  </div>
                </div>
                <div class="res-bar-item">
                  <div class="res-bar-head">
                    <span class="res-bar-label">RAM</span>
                    <span class="res-bar-pct">{{ (resourceMap[d.id].total_memory ? ((resourceMap[d.id].total_memory - resourceMap[d.id].free_memory) / resourceMap[d.id].total_memory * 100).toFixed(0) : 0) }}%</span>
                  </div>
                  <div class="res-bar-track">
                    <div class="res-bar-fill mem-fill" :style="{ width: (resourceMap[d.id].total_memory ? ((resourceMap[d.id].total_memory - resourceMap[d.id].free_memory) / resourceMap[d.id].total_memory * 100) : 0) + '%' }"></div>
                  </div>
                </div>
                <div class="res-bar-item">
                  <div class="res-bar-head">
                    <span class="res-bar-label">Disk</span>
                    <span class="res-bar-pct">{{ (resourceMap[d.id].total_hdd_space ? ((resourceMap[d.id].total_hdd_space - resourceMap[d.id].free_hdd_space) / resourceMap[d.id].total_hdd_space * 100).toFixed(0) : 0) }}%</span>
                  </div>
                  <div class="res-bar-track">
                    <div class="res-bar-fill disk-fill" :style="{ width: (resourceMap[d.id].total_hdd_space ? ((resourceMap[d.id].total_hdd_space - resourceMap[d.id].free_hdd_space) / resourceMap[d.id].total_hdd_space * 100) : 0) + '%' }"></div>
                  </div>
                </div>
              </div>
            </template>
            <template v-else>
              <div class="res-divider"></div>
              <div class="res-na">Resource tidak tersedia</div>
            </template>
          </div>
        </div>
      </div>

      <!-- Empty state -->
      <div v-if="devices.length === 0" class="empty">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="2" y="14" width="20" height="7" rx="2"/>
          <path d="M6 18h.01M10 18h.01"/>
          <path d="M12 3v4M8 7h8M7 7l5-4 5 4"/>
        </svg>
        <p>Belum ada perangkat MikroTik terdaftar.</p>
      </div>
    </template>
  </div>
</template>

<style scoped>
.home { max-width: 960px; }

/* Loading */
.loading {
  display: flex; justify-content: center; padding: var(--space-16);
  color: var(--color-text-secondary);
}
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
.spin { animation: spin 1s linear infinite; }

/* Header */
.page-title {
  font-size: var(--font-size-3xl);
  font-weight: 800;
  margin-bottom: var(--space-2);
}
.page-sub {
  font-size: var(--font-size-base);
  color: var(--color-text-secondary);
  margin-bottom: var(--space-8);
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
  margin-bottom: var(--space-10);
}
.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-6);
}
.stat-icon {
  width: 48px; height: 48px;
  display: flex; align-items: center; justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}
.stat-icon.blue { background: rgba(59,130,246,0.12); color: var(--color-blue); border: 1px solid rgba(59,130,246,0.2); }
.stat-icon.cyan { background: rgba(6,182,212,0.12); color: var(--color-cyan); border: 1px solid rgba(6,182,212,0.2); }
.stat-icon.violet { background: rgba(139,92,246,0.12); color: var(--color-violet); border: 1px solid rgba(139,92,246,0.2); }
.stat-number {
  display: block;
  font-size: var(--font-size-xl);
  font-weight: 800;
  color: var(--color-text-primary);
}
.stat-label {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* Aggregate Resource Summary Section */
.resource-summary-section { margin-bottom: var(--space-8); }
.res-summary-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: var(--space-4);
}
.res-summary-card {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-5);
}
.res-summary-icon {
  width: 44px; height: 44px;
  display: flex; align-items: center; justify-content: center;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}
.res-summary-icon.cpu { background: rgba(249,115,22,0.12); color: var(--color-orange); border: 1px solid rgba(249,115,22,0.2); }
.res-summary-icon.mem { background: rgba(139,92,246,0.12); color: var(--color-violet); border: 1px solid rgba(139,92,246,0.2); }
.res-summary-icon.disk { background: rgba(6,182,212,0.12); color: var(--color-cyan); border: 1px solid rgba(6,182,212,0.2); }
.res-summary-body { flex: 1; min-width: 0; }
.res-summary-label {
  display: block;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  margin-bottom: var(--space-2);
}
.res-summary-value {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: 700;
  color: var(--color-text-primary);
  font-family: monospace;
  margin-top: var(--space-1);
}

/* Resource Progress Bar (shared) */
.res-bar-track {
  height: 6px;
  background: var(--color-surface);
  border-radius: 3px;
  overflow: hidden;
}
.res-bar-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.4s ease;
}
.cpu-fill { background: var(--color-orange); }
.mem-fill { background: var(--color-violet); }
.disk-fill { background: var(--color-cyan); }

/* Per-Device Resource Bars */
.res-divider {
  height: 1px;
  background: var(--color-border);
  margin: var(--space-3) 0;
}
.res-bars { display: flex; flex-direction: column; gap: var(--space-2); }
.res-bar-head {
  display: flex;
  justify-content: space-between;
  margin-bottom: var(--space-1);
}
.res-bar-label {
  font-size: 11px;
  font-weight: 600;
  color: var(--color-text-muted);
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.res-bar-pct {
  font-size: 11px;
  font-weight: 700;
  color: var(--color-text-secondary);
  font-family: monospace;
}
.res-na {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  text-align: center;
  padding: var(--space-2);
  font-style: italic;
}

/* Bandwidth Section */
.bandwidth-section { margin-top: var(--space-4); }
.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-5);
}
.section-title {
  font-size: var(--font-size-xl);
  font-weight: 700;
}
.total-bw {
  display: flex;
  gap: var(--space-5);
  font-size: var(--font-size-sm);
  font-weight: 600;
}
.total-rx { color: var(--color-blue); }
.total-tx { color: var(--color-emerald); }

/* Bandwidth Card Grid */
.bw-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}
.bw-card {
  padding: var(--space-5);
  cursor: pointer;
  transition: transform 0.15s, box-shadow 0.15s;
}
.bw-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 30px rgba(0,0,0,0.15);
}

.bw-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-4);
  padding-bottom: var(--space-3);
  border-bottom: 1px solid var(--color-border);
}
.bw-name {
  font-weight: 700;
  font-size: var(--font-size-base);
  color: var(--color-text-primary);
}
.bw-host {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  font-family: monospace;
}

.bw-metrics { margin-bottom: var(--space-2); }
.bw-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}
.bw-metric {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--color-text-secondary);
}
.bw-metric svg { flex-shrink: 0; }
.bw-metric.down svg { color: var(--color-blue); }
.bw-metric.up svg { color: var(--color-emerald); }
.bw-label {
  display: block;
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}
.bw-value {
  display: block;
  font-size: var(--font-size-sm);
  font-weight: 700;
  color: var(--color-text-primary);
  font-family: monospace;
}

.bw-uptime {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-bottom: var(--space-3);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  font-family: monospace;
}
.bw-uptime svg { flex-shrink: 0; color: var(--color-emerald); opacity: 0.7; }

/* Sparkline 24h */
.sparkline-wrap {
  margin-bottom: var(--space-3);
  padding: var(--space-2) 0;
}
.sparkline-svg {
  display: block;
  overflow: visible;
}
.spark-line {
  fill: none;
  stroke-width: 1.5;
  stroke-linecap: round;
  stroke-linejoin: round;
}
.spark-line-rx { stroke: var(--color-blue); }
.spark-line-tx { stroke: var(--color-emerald); }
.spark-area { opacity: 0.1; }
.spark-area-rx { fill: var(--color-blue); }
.spark-area-tx { fill: var(--color-emerald); }
.sparkline-label {
  display: flex;
  justify-content: center;
  gap: var(--space-4);
  margin-top: var(--space-1);
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
}
.spark-rx { color: var(--color-blue); }
.spark-tx { color: var(--color-emerald); }

.bw-interface {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  font-family: monospace;
}
.bw-status-dot {
  width: 8px; height: 8px;
  border-radius: 50%;
  flex-shrink: 0;
}
.bw-status-dot.online { background: var(--color-emerald); box-shadow: 0 0 6px rgba(52,211,153,0.5); }
.bw-status-dot.offline { background: var(--color-red); }

.bw-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-6);
  color: var(--color-text-muted);
  font-size: var(--font-size-sm);
}
.bw-empty svg { opacity: 0.4; }

/* Tooltip */
.bw-card { position: relative; }
.bw-tooltip {
  display: none;
  position: absolute;
  bottom: calc(100% + 12px);
  left: 50%;
  transform: translateX(-50%);
  z-index: 100;
  min-width: 280px;
  max-width: 320px;
}
.bw-card:hover .bw-tooltip {
  display: block;
  animation: ttFadeIn 0.15s ease-out;
}
@keyframes ttFadeIn {
  from { opacity: 0; transform: translateX(-50%) translateY(6px); }
  to { opacity: 1; transform: translateX(-50%) translateY(0); }
}
.bw-tooltip-arrow {
  position: absolute;
  bottom: -6px;
  left: 50%;
  transform: translateX(-50%) rotate(45deg);
  width: 12px; height: 12px;
  background: #1a1a2e;
  border: 1px solid rgba(255,255,255,0.08);
  border-top: none;
  border-left: none;
}
.bw-tooltip-body {
  background: #1a1a2e;
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: var(--radius-lg);
  padding: var(--space-4);
  box-shadow: 0 16px 48px rgba(0,0,0,0.5);
  line-height: 1.5;
}
.tt-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: var(--space-4);
  padding: 2px 0;
}
.tt-label {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
}
.tt-val {
  font-size: var(--font-size-xs);
  font-weight: 600;
  color: var(--color-text-primary);
  text-align: right;
}
.tt-val.mono { font-family: monospace; }
.tt-rx { color: var(--color-blue); }
.tt-tx { color: var(--color-emerald); }
.tt-cpu { color: var(--color-orange); }
.tt-mem { color: var(--color-violet); }
.tt-disk { color: var(--color-cyan); }
.tt-warn { color: var(--color-red) !important; }
.tt-online { color: var(--color-emerald); }
.tt-offline { color: var(--color-red); }
.tt-divider {
  height: 1px;
  background: var(--color-border);
  margin: var(--space-2) 0;
}
.tt-section-title {
  font-size: 10px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.08em;
  color: var(--color-text-muted);
  margin-bottom: var(--space-1);
}
.tt-na {
  font-size: var(--font-size-xs);
  color: var(--color-text-muted);
  font-style: italic;
  padding: var(--space-1) 0;
}
.tt-click-hint {
  font-size: 10px;
  color: var(--color-text-muted);
  opacity: 0.6;
  text-align: center;
  margin-top: var(--space-1);
}

/* Empty State */
.empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-16) 0;
  color: var(--color-text-muted);
}

/* Responsive */
@media (max-width: 900px) {
  .res-summary-grid { grid-template-columns: 1fr; }
}
@media (max-width: 768px) {
  .stats-grid { grid-template-columns: 1fr; }
  .section-header { flex-direction: column; align-items: flex-start; gap: var(--space-2); }
  .bw-grid { grid-template-columns: 1fr; }
}
</style>
