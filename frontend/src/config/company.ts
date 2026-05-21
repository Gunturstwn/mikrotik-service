// ============================================================
// ⭐ FILE INI ADALAH PUSAT KONFIGURASI KONTEN PERUSAHAAN
// Edit file ini untuk mengubah semua konten di landing page
// tanpa perlu menyentuh komponen Vue
// ============================================================

export const company = {
  // ─── Info Dasar ──────────────────────────────────────────
  name: 'PC24 Telekomunikasi Indonesia',
  shortName: 'PC24Telin',
  tagline: 'Membangun Indonesia dengan Teknologi',
  subTagline: 'Penyedia layanan dan infrastruktur telekomunikasi sejak 2012. Mendukung transformasi digital nasional dengan solusi yang andal dan efisien.',
  foundedYear: 2012,
  logo: '/logo.svg', // letakkan file logo di folder public/

  // ─── Tombol CTA di Hero ───────────────────────────────────
  ctaPrimary: { label: 'Lihat Layanan Kami', href: '#services' },
  ctaSecondary: { label: 'Hubungi Kami', href: '#contact' },

  // ─── Statistik (ditampilkan di Stats Section) ─────────────
  stats: [
    { value: 500, suffix: '+', label: 'Klien Aktif' },
    { value: 34, suffix: '', label: 'Kota Jangkauan' },
    { value: 99.9, suffix: '%', label: 'Uptime SLA', isFloat: true },
    { value: 12, suffix: '+', label: 'Tahun Pengalaman' },
  ],

  // ─── Tentang Perusahaan ───────────────────────────────────
  about: {
    title: 'Tentang PC24',
    paragraphs: [
      'PT PC24 Telekomunikasi Indonesia (PC24Telin) adalah perusahaan yang bergerak di bidang layanan dan infrastruktur telekomunikasi, berkomitmen untuk menjadi mitra strategis dalam mendukung transformasi digital di Indonesia.',
      'Kami membuka diri untuk menjalin kerja sama dengan berbagai pihak, termasuk BUMN, instansi pemerintah, dan perusahaan swasta, guna membangun ekosistem telekomunikasi yang andal dan berkelanjutan.',
      'Melalui kolaborasi dan sinergi, PC24Telin hadir untuk mendukung pemerataan akses dan pengembangan infrastruktur telekomunikasi hingga ke seluruh penjuru wilayah NKRI.',
    ],
    vision: 'Menjadi perusahaan telekomunikasi terdepan yang mendukung pemerataan akses digital di seluruh Indonesia.',
    mission: [
      'Menyediakan infrastruktur telekomunikasi yang andal dan efisien',
      'Berkolaborasi dengan BUMN, pemerintah, dan sektor swasta',
      'Mendukung transformasi digital nasional secara menyeluruh',
      'Mensosialisasikan manfaat teknologi informasi kepada masyarakat',
    ],
  },

  // ─── Layanan ──────────────────────────────────────────────
  services: [
    {
      id: 'fiber',
      icon: '🔗',
      title: 'Koneksi Fiber Optik',
      description: 'Koneksi internet berkecepatan tinggi melalui infrastruktur fiber optik untuk bisnis dan enterprise.',
    },
    {
      id: 'wireless',
      icon: '📡',
      title: 'Koneksi Wireless',
      description: 'Solusi jaringan nirkabel yang fleksibel dan handal untuk area yang sulit dijangkau kabel.',
    },
    {
      id: 'satellite',
      icon: '🛰️',
      title: 'Koneksi Satelit',
      description: 'Layanan internet via satelit untuk daerah terpencil dan remote yang membutuhkan konektivitas.',
    },
    {
      id: 'broadband',
      icon: '🌐',
      title: 'Koneksi Broadband',
      description: 'Paket broadband scalable untuk kebutuhan residensial hingga korporat dengan bandwidth dedicated.',
    },
    {
      id: 'security',
      icon: '🛡️',
      title: 'Network Security',
      description: 'Solusi keamanan jaringan komprehensif: firewall, IDS/IPS, VPN, dan monitoring 24/7.',
    },
    {
      id: 'datacenter',
      icon: '🏢',
      title: 'Kolokasi Data Center',
      description: 'Fasilitas colocation dengan tier enterprise, redundant power, dan pendingin berstandar internasional.',
    },
    {
      id: 'software',
      icon: '💻',
      title: 'Software Development',
      description: 'Pengembangan solusi perangkat lunak custom untuk mendukung operasional bisnis Anda.',
    },
    {
      id: 'callcenter',
      icon: '📞',
      title: 'Jasa Call Center',
      description: 'Layanan call center profesional 24/7 dengan agen terlatih untuk kepuasan pelanggan Anda.',
    },
  ],

  // ─── Jangkauan / Coverage ────────────────────────────────
  coverage: {
    title: 'Jangkauan Kami',
    subtitle: 'Terhubung dengan operator terkemuka untuk memberikan performa dan layanan terbaik.',
    highlights: [
      { icon: '🗺️', label: '34 Provinsi', desc: 'Jangkauan nasional' },
      { icon: '🔌', label: 'Multi-Operator', desc: 'Interkoneksi dengan operator' },
      { icon: '⚡', label: '24/7 NOC', desc: 'Monitoring non-stop' },
      { icon: '🏗️', label: 'Infrastruktur', desc: 'Backbone sendiri' },
    ],
    operators: ['Telkom', 'Indosat', 'XL Axiata', 'Smartfren', 'Biznet', 'Icon+'],
  },

  // ─── Partner ──────────────────────────────────────────────
  partners: {
    title: 'Partner Kami',
    subtitle: 'Bermitra dengan perusahaan teknologi terkemuka dunia untuk solusi terbaik.',
    list: [
      { name: 'Cisco', logo: '' },
      { name: 'Huawei', logo: '' },
      { name: 'MikroTik', logo: '' },
      { name: 'Fortinet', logo: '' },
      { name: 'Dell', logo: '' },
      { name: 'VMware', logo: '' },
    ],
  },

  // ─── Kontak ───────────────────────────────────────────────
  contact: {
    title: 'Hubungi Kami',
    subtitle: 'Siap membantu Anda. Hubungi tim kami untuk konsultasi gratis.',
    address: 'Jl. Telekomunikasi No. 24, Jakarta Selatan, DKI Jakarta 12345',
    phone: '+62 21 1234 5678',
    whatsapp: '+62 812 3456 7890',
    email: 'info@pc24telin.com',
    businessEmail: 'business@pc24telin.com',
    operationalHours: 'Senin – Jumat: 08.00 – 17.00 WIB',
    emergencySupport: 'NOC 24/7: +62 21 9876 5432',
  },

  // ─── Sosial Media ─────────────────────────────────────────
  social: {
    linkedin: 'https://linkedin.com/company/pc24telin',
    twitter: 'https://twitter.com/pc24telin',
    instagram: 'https://instagram.com/pc24telin',
    youtube: 'https://youtube.com/@pc24telin',
  },

  // ─── Footer ───────────────────────────────────────────────
  footer: {
    copyright: `© ${new Date().getFullYear()} PT PC24 Telekomunikasi Indonesia. All rights reserved.`,
    tagline: 'Membangun Indonesia dengan Teknologi.',
  },
} as const
