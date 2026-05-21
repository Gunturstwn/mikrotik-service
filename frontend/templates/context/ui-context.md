# UI Context

## Theme

Dark only. Tidak ada light mode. Design language adalah dark technical workspace — background near-black berlapis, glassmorphism cards, dan gradient aksen cyan-blue untuk elemen interaktif. Kesan premium dan modern untuk perusahaan telekomunikasi.

## Colors

| Role | CSS Variable | Value |
|------|-------------|-------|
| Page background | `--color-bg` | `#060b14` |
| Secondary background | `--color-bg-secondary` | `#0d1829` |
| Card background | `--color-bg-card` | `rgba(255, 255, 255, 0.04)` |
| Card hover | `--color-bg-card-hover` | `rgba(255, 255, 255, 0.07)` |
| Border | `--color-border` | `rgba(255, 255, 255, 0.08)` |
| Border hover | `--color-border-hover` | `rgba(6, 182, 212, 0.4)` |
| Primary text | `--color-text-primary` | `#f1f5f9` |
| Secondary text | `--color-text-secondary` | `#94a3b8` |
| Muted text | `--color-text-muted` | `#64748b` |
| Cyan accent | `--color-cyan` | `#06b6d4` |
| Cyan light | `--color-cyan-light` | `#22d3ee` |
| Blue accent | `--color-blue` | `#3b82f6` |
| Violet accent | `--color-violet` | `#8b5cf6` |
| Emerald | `--color-emerald` | `#10b981` |

## Gradients

| Name | Variable | Usage |
|------|----------|-------|
| Primary | `--gradient-primary` | Buttons, top lines, accents |
| Hero | `--gradient-hero` | Hero section background |
| Text | `--gradient-text` | Gradient text headings |
| Card | `--gradient-card` | Subtle card backgrounds |

## Typography

| Role | Font | Variable |
|------|------|----------|
| UI text | Inter | `--font-family` |
| Sizes | 0.75rem – 4.5rem | `--font-size-xs` to `--font-size-7xl` |

Scale: xs (0.75rem), sm (0.875rem), base (1rem), lg (1.125rem), xl (1.25rem), 2xl (1.5rem), 3xl (1.875rem), 4xl (2.25rem), 5xl (3rem), 6xl (3.75rem), 7xl (4.5rem)

## Border Radius

| Context | Variable | Value |
|---------|----------|-------|
| Small UI | `--radius-sm` | 0.375rem |
| Medium | `--radius-md` | 0.75rem |
| Large | `--radius-lg` | 1rem |
| Extra large | `--radius-xl` | 1.5rem |
| 2XL | `--radius-2xl` | 2rem |
| Pills/badges | `--radius-full` | 9999px |

## Component Library

Tidak menggunakan library UI eksternal. Semua komponen custom-built. Utility classes global didefinisikan di `main.css`:

- `.glass-card` — Card dengan backdrop blur dan border translucent
- `.btn` / `.btn-primary` / `.btn-secondary` — Button styles
- `.section-label` — Badge label section (uppercase, cyan)
- `.section-title` — Heading section (clamp responsive)
- `.section-subtitle` — Subtitle section (secondary color)
- `.gradient-text` — Text dengan gradient background clip
- `.container` — Max-width 1280px centered
- `.section` — Padding section (responsive)
- `.divider` — Horizontal gradient line
- `.form-input` / `.form-textarea` — Form elements
- `.animate-fade-up` / `.animate-float` — Animation utilities

## Layout Patterns

- Navbar: Fixed top, 72px height, blur background on scroll, hamburger di mobile
- Hero: Full viewport height, centered content, animated orbs background
- Sections: Padding 8rem vertical (5rem di mobile), max-width 1280px
- Cards: Glass morphism (backdrop-filter blur, translucent border)
- Grids: CSS Grid responsive (4 cols → 2 cols → 1 col)
- Footer: 4-column grid (brand + 3 nav columns)

## Icons

Inline SVG. Stroke-based icons. Sizes: 14px–20px untuk inline, 24px untuk feature icons. Tidak menggunakan icon library — semua SVG di-inline langsung di template.

## Animations

- `fadeInUp` — Fade in dari bawah (sections on scroll)
- `float` — Floating gentle movement (logo, orbs)
- `pulse-ring` — Expanding ring pulse (badge dots, map markers)
- `shimmer` — Background position animation (scroll indicator)
- `spin` — Loading spinner rotation
- Transitions: fast (150ms), base (250ms), slow (400ms)

## Responsive Breakpoints

- Desktop: > 1100px (full grid layouts)
- Tablet: 768px – 1100px (reduced columns)
- Mobile: < 768px (single column, hamburger menu)
- Small mobile: < 480px (compact spacing)
