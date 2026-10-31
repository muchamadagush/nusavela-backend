# AGENTS.md - Panduan Arsitektur & Konvensi AI Coding Agent untuk NusaVela (`nusavela.com`)

File ini berfungsi sebagai instruksi operasional wajib bagi seluruh *AI Coding Agents* (Cursor, Claude Code, Windsurf, GitHub Copilot, Gemini) dan *developer* yang bekerja di repositori **NusaVela (`nusavela.com`)**.

## 1. Konteks Proyek & Tech Stack Utama
**NusaVela** adalah platform direktori geospasial, wisata, fasilitas kesehatan (IGD 24 jam), transportasi, dan berita daerah untuk **514 Kabupaten/Kota di seluruh Indonesia**.

* **Frontend Web:** [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview) + [TanStack Router](https://tanstack.com/router/latest/docs/guide/search-params) + TanStack Query + Tailwind CSS + [Leaflet.js](https://leafletjs.com/) / MapLibre GL.
* **Backend API & Data Pipeline:** **Rust** ([Axum 0.8+](https://crates.io/crates/axum), [Tokio](https://crates.io/crates/tokio), [SQLx 0.8+](https://crates.io/crates/sqlx), [osmpbf](https://crates.io/crates/osmpbf), [feed-rs](https://crates.io/crates/feed-rs/code)).
* **Database:** **PostgreSQL + PostGIS** (dengan ekstensi `postgis` dan `uuid-ossp`).
* **Map Vector Tile Server:** [Martin Tile Server (Rust)](https://github.com/maplibre/martin) untuk penyajian peta mandiri yang mematuhi [OSM Tile Usage Policy](https://operations.osmfoundation.org/policies/tiles/).

## 2. Struktur Monorepo yang Wajib Dipatuhi

```text
nusavela/
├── AGENTS.md                  # Instruksi operasional agen AI
├── DESIGN.md                  # Spesifikasi Design System & Color Palette NusaVela
├── SKILL.md                   # Definisi alur kerja pembuatan fitur & halaman kota
├── migrations/                # File SQL migrasi PostgreSQL + PostGIS (dikelola sqlx-cli)
├── backend/                   # Workspace Cargo Rust (Axum API + CLI Seeders)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs            # Entrypoint Axum HTTP Server + Tokio Cron Worker
│       ├── routes/            # Handler endpoint (/api/v1/kota/..., /api/v1/submissions)
│       ├── models/            # Struct Serde + SQLx FromRow
│       ├── workers/           # Background RSS parser (feed-rs) & cache cuaca
│       └── bin/
│           ├── seed_wilayah.rs # Impor 38 provinsi & 514 kab/kota
│           └── seed_osm.rs     # Parallel parser indonesia-latest.osm.pbf (osmpbf)
└── frontend/                  # Aplikasi Web SSR TanStack Start
    ├── package.json
    └── src/
        ├── routes/
        │   ├── __root.tsx
        │   ├── index.tsx      # Beranda Nasional + GPS Detector + 514 City Combobox
        │   └── kota/
        │       └── $citySlug/
        │           ├── index.tsx               # Bento Dashboard Kota
        │           └── $categorySlug/
        │               ├── index.tsx           # Split-View Peta & Direktori Kategori
        │               └── $placeSlug.tsx      # Halaman Detail Lokasi + CTA Affiliate
        ├── components/
        │   ├── map/            # Client-only MapLibre / Leaflet components
        │   ├── cards/          # PlaceCard, SponsoredCard, EmergencyBanner
        │   └── seo/            # JSON-LD Schema generators (TouristAttraction, Hospital)
        └── styles/
            └── app.css         # Token CSS sesuai DESIGN.md
```

## 3. Aturan Koding Backend Rust (`backend/`)

1. **Zero `unwrap()` di Jalur HTTP Handler:**
   * Seluruh *handler* [Axum](https://crates.io/crates/axum) wajib mengembalikan `Result<Json<T>, AppError>`. Jangan pernah menggunakan `.unwrap()` atau `.expect()` pada input pengguna atau hasil query database.
2. **Query Geospasial PostGIS & Urutan Monetisasi:**
   * Setiap *query* daftar tempat (`places`) wajib mengurutkan item mitra berbayar terlebih dahulu: `ORDER BY p.is_sponsored DESC, p.sponsored_tier DESC, distance_meters ASC NULLS LAST`.
   * Gunakan indeks `GIST` melalui fungsi `ST_DWithin(p.geom, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $radius)` untuk filter radius lokasi terdekat.
3. **Kepatuhan Hak Cipta RSS Berita (`workers/rss.rs`):**
   * Saat mem-parsing RSS menggunakan [feed-rs](https://crates.io/crates/feed-rs/code), potong (*truncate*) ringkasan berita maksimal **160–200 karakter** dan wajib simpan `source_url` asli serta `source_name`. Dilarang menyimpan atau menyajikan isi artikel berita penuh.

## 4. Aturan Koding Frontend TanStack Start (`frontend/`)

1. **Validasi Search Params dengan Zod (`validateSearch`):**
   * Setiap filter direktori dan peta (`lat`, `lng`, `radius`, `open24h`, `sort`, `page`) wajib didefinisikan di dalam `validateSearch` pada `createFileRoute` sesuai panduan [TanStack Router Search Params](https://tanstack.com/router/latest/docs/guide/search-params). Jangan menggunakan `window.location.search` manual.
2. **Programmatic SEO & Caching ISR di Setiap Rute Kota:**
   * Setiap rute di bawah `/kota/$citySlug` wajib mengimplementasikan:
     * `headers()`: Mengembalikan `'Cache-Control': 'public, max-age=300, s-maxage=3600, stale-while-revalidate=86400'` untuk [TanStack Start ISR](https://tanstack.com/start/latest/docs/framework/react/guide/isr).
     * `head({ loaderData })`: Menghasilkan `<title>`, `<meta name="description">`, *Open Graph*, dan *JSON-LD structured data* dinamis berdasarkan nama kota dan kategori ([TanStack Start SEO](https://tanstack.com/start/latest/docs/framework/react/guide/seo)).
3. **Kepatuhan Desain & Etika Iklan (`DESIGN.md`):**
   * Gunakan variabel warna dari `DESIGN.md` (`#FAF8F5` untuk latar kanvas, `#0B1F3A` untuk *Deep Vela Navy*, `#0D5C46` untuk *Emerald Nusantara*, dan `#C89B3C` untuk lencana *Swarna Gold* pada UMKM berbayar).
   * **Aturan Keras Keselamatan Publik:** Dilarang merender komponen iklan AdSense atau *pop-up* afiliasi di atas komponen `EmergencyBanner` atau di halaman darurat medis/SOS.
   * Wajib menampilkan atribusi `© OpenStreetMap contributors` pada setiap komponen peta dan atribusi OpenWeather pada komponen cuaca.

## 5. Perintah Verifikasi & Pengujian Sebelum Commit

* **Cek Kompilasi & Linter Rust:**
  ```bash
  cd backend && cargo check && cargo clippy -- -D warnings && cargo test
  ```
* **Cek Tipe TypeScript & Build TanStack Start:**
  ```bash
  cd frontend && npm run typecheck && npm run build
  ```
