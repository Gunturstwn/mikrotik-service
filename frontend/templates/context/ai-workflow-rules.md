# AI Workflow Rules

## Approach

Build project ini secara inkremental menggunakan spec-driven workflow. Context files mendefinisikan apa yang dibangun, bagaimana membangunnya, dan status progress saat ini. Selalu implementasi berdasarkan specs ini — jangan infer atau invent behavior dari scratch.

## Scoping Rules

- Kerjakan satu feature unit pada satu waktu
- Prefer small, verifiable increments daripada large speculative changes
- Jangan combine unrelated system boundaries dalam satu implementation step

## When to Split Work

Split implementation step jika menggabungkan:

- UI changes dan API layer changes
- Multiple unrelated page/section modifications
- Behavior yang belum didefinisikan di context files
- Styling system changes dan component logic changes

Jika sebuah change tidak bisa diverifikasi end-to-end dengan cepat, scope terlalu broad — split.

## Handling Missing Requirements

- Jangan invent product behavior yang tidak didefinisikan di context files
- Jika requirement ambigu, resolve di relevant context file sebelum implementing
- Jika requirement missing, tambahkan sebagai open question di `progress-tracker.md`

## Protected Files

Jangan modify file berikut kecuali explicitly instructed:

- `src/assets/main.css` — Design system global (perubahan berdampak luas)
- `src/config/company.ts` — Konten perusahaan (hanya diubah oleh product owner)
- `node_modules/` — Dependencies (managed by npm)

## Keeping Docs in Sync

Update relevant context file ketika implementation mengubah:

- System architecture atau boundaries
- Storage model decisions
- Code conventions atau standards
- Feature scope
- UI patterns atau design tokens

## Before Moving to the Next Unit

1. Current unit works end-to-end dalam defined scope
2. Tidak ada invariant dari `architecture.md` yang dilanggar
3. `progress-tracker.md` mencerminkan completed work
4. `npm run build` passes tanpa error
5. Responsive layout tetap berfungsi di mobile dan desktop
