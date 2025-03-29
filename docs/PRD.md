# Blueprint & Product Requirements Document (PRD): Platform Informasi Lokal Indonesia (514 Kabupaten/Kota)
**Tech Stack Utama:** Frontend **TanStack Start** + Backend **Rust (Axum, SQLx, osmpbf)** + Database **PostgreSQL (PostGIS)**

Dokumen ini memuat spesifikasi lengkap pengembangan aplikasi dan situs web **Informasi Lokal Indonesia (Local Discovery & Public Utility Platform)** berbasis model hibrida (*Programmatic SEO* untuk 514 Kabupaten/Kota sekaligus + kurasi mendalam untuk kota *Tier-1*). Seluruh bagian dokumen ini telah diselaraskan sepenuhnya dengan arsitektur terbaru menggunakan **TanStack Start** di sisi *front-end* dan **Rust** di sisi *backend* serta *data pipeline*.

## 1. Product Requirements Document (PRD)

### 1.1 Ringkasan Eksekutif & Tujuan Produk
Platform ini adalah portal informasi lokal terpadu berbasis web ([TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview)) dan aplikasi mobile ([Flutter](https://www.google.com/search?q=https://flutter.dev/)) yang didukung oleh *backend* berkinerja tinggi berbasis **Rust** ([Axum](https://crates.io/crates/axum)). Platform menyajikan informasi tempat wisata, fasilitas kesehatan (RS/IGD 24 jam, klinik, apotek), rute transportasi publik, berita daerah terkini, agenda *event* lokal, serta nomor darurat di seluruh Indonesia.

* **Masalah Utama:** Informasi daerah di Indonesia masih terfragmentasi. Wisatawan kesulitan mencari harga tiket dan rute angkutan lokal dalam satu tempat, sementara warga lokal kesulitan menemukan daftar IGD 24 jam, nomor darurat daerah, atau agenda kegiatan akhir pekan yang terverifikasi.
* **Solusi:** Menggabungkan *database* geospasial terbuka seluruh Indonesia ([OpenStreetMap](https://www.google.com/search?q=https://www.openstreetmap.org/copyright)) yang diekstrak secara paralel menggunakan Rust ([osmpbf](https://crates.io/crates/osmpbf)), agregasi berita lokal (RSS via [feed-rs](https://crates.io/crates/feed-rs/code)), informasi cuaca ([OpenWeatherMap API](https://www.google.com/search?q=https://openweathermap.org/api)), serta kontribusi komunitas (*crowdsourcing*) dalam antarmuka SSR cepat berbasis [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview).
* **Strategi Ekspansi Nasional (Hybrid Approach):**
  1. **Cakupan Otomatis Nasional (514 Kab/Kota):** Meng-*generate* halaman direktori otomatis untuk 38 provinsi dan 514 kabupaten/kota menggunakan data terbuka dari [API Wilayah Indonesia (emsifa)](https://github.com/emsifa/api-wilayah-indonesia) dan ekstrak file PBF seluruh Indonesia dari [Geofabrik Indonesia](https://download.geofabrik.de/asia/indonesia.html).
  2. **Kurasi Mendalam Kota Tier-1:** Memperkaya data harga tiket, jadwal transportasi lokal, dan kemitraan UMKM pada kota-kota prioritas (misalnya Bandung, Yogyakarta, Surakarta/Sragen, Malang, Surabaya, dan Denpasar).

### 1.2 Ringkasan Arsitektur Teknologi Terbaru

| Lapisan Sistem | Teknologi yang Digunakan | Alasan Pemilihan untuk Skala 514 Kota |
| :--- | :--- | :--- |
| **Frontend Web (SSR & SEO)** | [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview) + [TanStack Router](https://tanstack.com/router/latest/docs/guide/search-params) + TanStack Query | Mendukung *Full-Document SSR* untuk SEO ([TanStack Start SEO](https://tanstack.com/start/latest/docs/framework/react/guide/seo)), *ISR Caching* berbasis HTTP Header standar ([TanStack Start ISR](https://tanstack.com/start/latest/docs/framework/react/guide/isr)), serta *100% Type-Safe Search Params* untuk filter koordinat GPS dan radius peta. |
| **Backend API & Worker** | **Rust** ([Axum](https://crates.io/crates/axum) + [Tokio](https://crates.io/crates/tokio) + [SQLx](https://crates.io/crates/sqlx)) | Konsumsi RAM sangat rendah (<30 MB saat *idle*), pemeriksaan *query* SQL/PostGIS saat kompilasi, serta mampu menggabungkan REST API dan *cron worker* berita RSS dalam satu *binary* tunggal. |
| **Data Seeder Se-Indonesia** | **Rust CLI** ([osmpbf](https://crates.io/crates/osmpbf)) | Mengekstrak ratusan ribu titik wisata, RS, apotek, dan stasiun dari file `indonesia-latest.osm.pbf` (±1,5 GB) secara paralel dalam 1–3 menit tanpa terkena *rate-limit* Overpass API. |
| **Database Geospasial** | **PostgreSQL + PostGIS** ([Supabase](https://www.google.com/search?q=https://supabase.com/pricing) / Self-Hosted) | Mendukung *spatial indexing* (`GIST`) dan perhitungan jarak terdekat (`ST_DWithin`, `ST_Distance`) dalam hitungan milidetik. |
| **Peta & Vector Tile Server** | [Leaflet.js](https://www.google.com/search?q=https://leafletjs.com/) / MapLibre + [Martin Tile Server (Rust)](https://github.com/maplibre/martin) | Menyajikan peta interaktif secara mandiri dari PostGIS/PMTiles sehingga bebas risiko pemblokiran server publik saat aplikasi dimonetisasi. |

### 1.3 Target Pengguna (User Personas)

| Persona | Profil Pengguna | Kebutuhan Utama | Potensi Monetisasi |
| :--- | :--- | :--- | :--- |
| **Wisatawan & Pelancong** | Pengunjung luar kota atau keluarga yang merencanakan liburan akhir pekan. | Mencari tempat wisata populer, harga tiket masuk, jam buka, rute transportasi, dan penginapan terdekat. | Komisi afiliasi hotel & tiket wisata ([Traveloka Affiliate](https://www.google.com/search?q=https://www.traveloka.com/id-id/p/affiliate), [Blibli Tiket Affiliate](https://www.google.com/search?q=https://www.tiket.com/id-id/explore/blibli-tiket-affiliate)), fitur *AI Itinerary*. |
| **Warga Lokal (Harian)** | Penduduk setempat yang membutuhkan layanan darurat, faskes, atau hiburan akhir pekan. | Mencari IGD 24 jam terdekat, apotek buka malam, nomor ambulans/damkar lokal, berita daerah, dan jadwal *Car Free Day* (CFD) / bazar. | Impresi iklan (*display ads* via [Google AdSense](https://support.google.com/adsense/answer/9234653?hl=en) / [Google AdMob](https://www.google.com/search?q=https://admob.google.com/home/)) dan promosi *event*. |
| **Pemilik UMKM & Penyelenggara Event** | Pemilik kafe, restoran, rental kendaraan, klinik swasta, dan EO lokal. | Meningkatkan visibilitas usaha di halaman pencarian kota dan mempromosikan acara kepada warga maupun wisatawan. | Langganan *Sponsored/Verified Listing* bulanan dan biaya penayangan *banner event*. |

### 1.4 Strategi Monetisasi & Kepatuhan Lisensi

| Jalur Pendapatan | Mekanisme Implementasi | Aturan Teknis & Legal yang Wajib Dipatuhi |
| :--- | :--- | :--- |
| **Affiliate Marketing (Wisata & Hotel)** | Tombol CTA *"Cek Harga Tiket & Hotel Terdekat"* pada setiap halaman detail wisata dan stasiun/terminal. | Gunakan parameter *tracking* resmi dari [Traveloka Affiliate](https://www.google.com/search?q=https://www.traveloka.com/id-id/p/affiliate) atau [Blibli Tiket Affiliate](https://www.google.com/search?q=https://www.tiket.com/id-id/explore/blibli-tiket-affiliate). |
| **Sponsored Business Listing (B2B)** | Penempatan urutan teratas (*pinned card*) dan lencana *Verified Partner* di peta maupun daftar kategori. | Tandai secara transparan dengan label *"Sponsor"* atau *"Rekomendasi Mitra"* agar tetap menjaga kepercayaan pengguna. |
| **Programmatic Display Ads** | Slot iklan responsif di antara daftar berita dan direktori wisata menggunakan [Google AdSense](https://support.google.com/adsense/answer/9234653?hl=en) (Web) & [Google AdMob](https://www.google.com/search?q=https://admob.google.com/home/) (App). | **Dilarang** memasang iklan *interstitial/pop-up* yang menghalangi halaman **Nomor Darurat & IGD Rumah Sakit**. |
| **Kepatuhan Peta & Data Cuaca** | Menampilkan peta interaktif dan widget cuaca kota. | Wajib mencantumkan atribusi `© OpenStreetMap contributors` ([OSM Legal FAQ](https://www.google.com/search?q=https://osmfoundation.org/wiki/Licence/Licence_and_Legal_FAQ)) dan atribusi OpenWeather ([OpenWeather Pricing \& License](https://www.google.com/search?q=https://openweathermap.org/full-price)). Gunakan [Martin Tile Server](https://github.com/maplibre/martin) atau penyedia *tile* komersial saat trafik tinggi sesuai [OSM Tile Usage Policy](https://www.google.com/search?q=https://operations.osmfoundation.org/policies/tiles/). |
| **Kepatuhan Agregator Berita (RSS)** | Menampilkan berita terbaru per kota/provinsi. | Hanya tampilkan judul, cuplikan singkat (maksimal 160 karakter), nama media, dan tautan langsung ke sumber asli untuk menghormati hak cipta penerbit. |

### 1.5 Metrik Keberhasilan (Key Performance Indicators)
* **Cakupan Indeks SEO:** >90% halaman kota dan kategori (`/kota/$citySlug/$categorySlug`) terindeks di Google dalam 60 hari pertama melalui Full-Document SSR & HTTP Cache ISR pada [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/guide/isr).
* **Performa API & Halaman:** Waktu respons *endpoint* geospasial Rust <10 milidetik dan skor *Core Web Vitals* (LCP < 2,0 detik).
* **Konversi Monetisasi:** *Click-Through Rate* (CTR) tombol afiliasi wisata/hotel minimal 3–5% dari total pengunjung halaman wisata, serta akuisisi minimal 10 UMKM mitra berbayar per kota *Tier-1*.

## 2. Daftar Fitur Lengkap (Feature List & Prioritas MoSCoW)

### 2.1 Modul Pengguna & Publik (TanStack Start Front-End)

| Prioritas | Nama Fitur | Deskripsi Fungsional | Cakupan Data Awal |
| :--- | :--- | :--- | :--- |
| **Must Have (MVP)** | **Deteksi Lokasi Pintar & Pemilih Kota** | Deteksi lokasi otomatis via Geolocation API yang memanggil *endpoint* PostGIS Rust (`/api/v1/cities/nearest`) disertai *combobox* pencarian 514 kabupaten/kota. | 514 Kab/Kota (Otomatis) |
| **Must Have (MVP)** | **Direktori Geospasial & Type-Safe Filter** | Halaman daftar & peta interaktif untuk kategori **Wisata, Rumah Sakit & Faskes, Kuliner, Transportasi, Layanan Publik**, dengan filter URL *type-safe* (`?lat=...&lng=...&radius=5000&open24h=true`) via [TanStack Router](https://tanstack.com/router/latest/docs/guide/search-params). | 514 Kab/Kota via Ekstrak PBF [Geofabrik Indonesia](https://download.geofabrik.de/asia/indonesia.html) |
| **Must Have (MVP)** | **Panel Darurat & Faskes 24 Jam (SOS)** | Akses cepat 1-klik (`tel:`) ke nomor darurat nasional (112, 119, 110, 113) dan daftar IGD Rumah Sakit terdekat beserta tombol navigasi rute langsung. | Nasional + RS per Kota |
| **Must Have (MVP)** | **Halaman Detail Lokasi & Integrasi Afiliasi** | Menampilkan nama tempat, alamat, jam buka, estimasi harga tiket, peta lokasi, cuaca sekitar, dan tombol afiliasi hotel/tiket terdekat. | 514 Kab/Kota |
| **Should Have** | **Agregator Berita Daerah (RSS)** | Menampilkan *headline* berita lokal terbaru berdasarkan kota/provinsi yang dipilih pengguna, diperbarui otomatis oleh *worker* Rust. | Per Provinsi / Kota Utama |
| **Should Have** | **Kalender Event & Agenda Kota** | Daftar acara lokal (konser, CFD, pameran, bazar UMKM) dengan filter tanggal (Hari Ini, Akhir Pekan Ini, Bulan Ini). | Kurasi Kota Tier-1 & Komunitas |
| **Should Have** | **Form Kontribusi Komunitas & Klaim Bisnis** | Tombol *"Koreksi Data / Tambah Tempat Baru"* serta formulir *"Promosikan Usaha Anda"* untuk pemilik UMKM yang dikirim ke *endpoint* Rust `POST /api/v1/submissions`. | 514 Kab/Kota |
| **Could Have** | **Info Rute Transportasi Publik Lokal** | Daftar koridor bus lokal (misal TMB Bandung, Batik Solo Trans, TransJogja), stasiun kereta, dan terminal beserta peta rutenya. | Bertahap mulai Kota Tier-1 |
| **Could Have** | **AI Local Guide & Itinerary Generator** | Chatbot asisten lokal yang menjawab pertanyaan wisata/kuliner berdasarkan data terverifikasi di *database* dan menyusun rencana perjalanan otomatis. | Freemium (Kuota Gratis + Berbayar) |

### 2.2 Modul Admin & Otomasi Backend (Rust Engine)
* **Parallel OSM PBF Seeder (`seed_osm` binary):** Program CLI berbasis crate [osmpbf](https://crates.io/crates/osmpbf) untuk mengekstrak ratusan ribu POI wisata, faskes, dan transportasi dari file `indonesia-latest.osm.pbf` langsung ke PostgreSQL.
* **Wilayah Master Seeder (`seed_wilayah` binary):** Program CLI Rust untuk mengimpor 38 provinsi dan 514 kabupaten/kota dari [API Wilayah Indonesia (emsifa)](https://github.com/emsifa/api-wilayah-indonesia).
* **Embedded Background Worker (RSS & Weather Cache):** Task asinkron di dalam server [Tokio](https://crates.io/crates/tokio) yang mengambil *feed* berita daerah setiap 2 jam menggunakan [feed-rs](https://crates.io/crates/feed-rs/code) dan membersihkan berita lawas secara otomatis.
* **Moderasi Kontribusi & Kampanye UMKM:** Endpoint admin terproteksi JWT untuk menyetujui klaim bisnis, mengaktifkan status `is_sponsored`, dan memantau analitik klik dari tabel `monetization_clicks`.

## 3. Struktur Database Lengkap (PostgreSQL + PostGIS & SQLx Migrations)

Skema berikut disimpan di folder `migrations/0001_initial_schema.sql` dan dijalankan melalui `sqlx migrate run` ([sqlx-cli](https://www.google.com/search?q=https://crates.io/crates/sqlx-cli)) pada database PostgreSQL (baik di [Supabase](https://www.google.com/search?q=https://supabase.com/pricing) maupun PostgreSQL mandiri):

```sql
-- 1. Aktifkan ekstensi PostGIS dan UUID
CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- 2. Tabel Master Provinsi (38 Provinsi)
CREATE TABLE provinces (
    id VARCHAR(10) PRIMARY KEY, -- Kode Kemendagri/BPS (contoh: '32' Jawa Barat, '33' Jawa Tengah, '35' Jawa Timur)
    name VARCHAR(100) NOT NULL,
    slug VARCHAR(120) UNIQUE NOT NULL,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    rss_news_urls TEXT[], -- Array URL RSS berita tingkat provinsi
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 3. Tabel Master Kabupaten/Kota (514 Kab/Kota)
CREATE TABLE cities (
    id VARCHAR(10) PRIMARY KEY, -- Kode Kemendagri/BPS (contoh: '3273' Kota Bandung, '3578' Kota Surabaya)
    province_id VARCHAR(10) NOT NULL REFERENCES provinces(id) ON DELETE CASCADE,
    name VARCHAR(150) NOT NULL, -- Contoh: 'Kota Bandung', 'Kota Surabaya'
    short_name VARCHAR(100) NOT NULL, -- Contoh: 'Bandung', 'Surabaya'
    slug VARCHAR(150) UNIQUE NOT NULL, -- Contoh: 'kota-bandung', 'kota-surabaya'
    tier SMALLINT NOT NULL DEFAULT 2, -- 1 = Kota Fokus Kurasi (Tier-1), 2 = Kota Otomatis Programmatic SEO
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    geom GEOGRAPHY(Point, 4326), -- Kolom geospasial untuk deteksi kota terdekat dari GPS user
    population INT,
    area_km2 DECIMAL(10, 2),
    hero_image_url TEXT,
    meta_description TEXT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Tabel Kategori Direktori
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL, -- Wisata, Rumah Sakit, Kuliner, Transportasi, Layanan Publik
    slug VARCHAR(100) UNIQUE NOT NULL, -- wisata, rumah-sakit, kuliner, transportasi, layanan-publik
    icon_name VARCHAR(50),
    osm_tags JSONB, -- Pemetaan tag OpenStreetMap, contoh: {"amenity": ["hospital", "clinic"]}
    sort_order SMALLINT NOT NULL DEFAULT 0
);

-- 5. Tabel Utama Direktori Lokasi (Places / POI)
CREATE TABLE places (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    osm_id BIGINT UNIQUE, -- ID asli dari OpenStreetMap (mencegah duplikasi saat sinkronisasi PBF)
    city_id VARCHAR(10) NOT NULL REFERENCES cities(id) ON DELETE CASCADE,
    category_id INT NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(300) NOT NULL,
    address TEXT,
    latitude DECIMAL(10, 8) NOT NULL,
    longitude DECIMAL(11, 8) NOT NULL,
    geom GEOGRAPHY(Point, 4326), -- Untuk query jarak terdekat (ST_DWithin / ST_Distance)
    phone VARCHAR(50),
    whatsapp VARCHAR(50),
    website_url TEXT,
    opening_hours VARCHAR(255), -- Format OSM atau teks (contoh: '24/7', '08:00-17:00')
    is_24_hours BOOLEAN NOT NULL DEFAULT FALSE, -- Filter penting untuk IGD Rumah Sakit & Apotek
    ticket_price_min INT,
    ticket_price_max INT,
    rating_avg DECIMAL(3, 2) NOT NULL DEFAULT 0.00,
    review_count INT NOT NULL DEFAULT 0,
    description TEXT,
    image_url TEXT,
    -- Kolom Monetisasi (Affiliate & Sponsored Listing B2B)
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    is_sponsored BOOLEAN NOT NULL DEFAULT FALSE,
    sponsored_until TIMESTAMPTZ,
    sponsored_tier SMALLINT NOT NULL DEFAULT 0, -- 0 = Gratis, 1 = Featured Card, 2 = Top Banner
    affiliate_hotel_url TEXT,
    affiliate_ticket_url TEXT,
    source VARCHAR(50) NOT NULL DEFAULT 'osm', -- 'osm', 'admin', 'community', 'partner'
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(city_id, slug)
);

-- 6. Tabel Nomor Darurat (Nasional & Spesifik Kota)
CREATE TABLE emergency_contacts (
    id SERIAL PRIMARY KEY,
    city_id VARCHAR(10) REFERENCES cities(id) ON DELETE CASCADE, -- NULL jika berlaku Nasional (112, 119, 110)
    service_name VARCHAR(150) NOT NULL,
    category VARCHAR(50) NOT NULL, -- 'medis', 'keamanan', 'pemadam', 'sar', 'call_center'
    phone_number VARCHAR(50) NOT NULL,
    is_toll_free BOOLEAN NOT NULL DEFAULT FALSE,
    is_24_hours BOOLEAN NOT NULL DEFAULT TRUE,
    sort_order SMALLINT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- 7. Tabel Event & Agenda Lokal
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    city_id VARCHAR(10) NOT NULL REFERENCES cities(id) ON DELETE CASCADE,
    title VARCHAR(255) NOT NULL,
    slug VARCHAR(300) NOT NULL,
    venue_name VARCHAR(255),
    address TEXT,
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    start_datetime TIMESTAMPTZ NOT NULL,
    end_datetime TIMESTAMPTZ NOT NULL,
    ticket_price INT NOT NULL DEFAULT 0,
    ticket_affiliate_url TEXT,
    organizer_name VARCHAR(150),
    poster_url TEXT,
    description TEXT,
    is_sponsored BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(city_id, slug)
);

-- 8. Tabel Agregator Berita Daerah (RSS Cache)
CREATE TABLE news_feeds (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    province_id VARCHAR(10) REFERENCES provinces(id) ON DELETE CASCADE,
    city_id VARCHAR(10) REFERENCES cities(id) ON DELETE CASCADE,
    title VARCHAR(300) NOT NULL,
    summary TEXT,
    source_name VARCHAR(100) NOT NULL,
    source_url TEXT UNIQUE NOT NULL,
    thumbnail_url TEXT,
    published_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 9. Tabel Kontribusi Komunitas & Klaim Bisnis UMKM (Crowdsourcing)
CREATE TABLE community_submissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    city_id VARCHAR(10) NOT NULL REFERENCES cities(id) ON DELETE CASCADE,
    place_id UUID REFERENCES places(id) ON DELETE SET NULL,
    submission_type VARCHAR(50) NOT NULL, -- 'new_place', 'correction', 'business_claim', 'new_event'
    submitter_name VARCHAR(100),
    submitter_contact VARCHAR(100),
    payload JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected'
    admin_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ
);

-- 10. Tabel Analitik Klik Monetisasi (Affiliate & Lead Tracking)
CREATE TABLE monetization_clicks (
    id BIGSERIAL PRIMARY KEY,
    city_id VARCHAR(10) REFERENCES cities(id) ON DELETE SET NULL,
    place_id UUID REFERENCES places(id) ON DELETE SET NULL,
    click_type VARCHAR(50) NOT NULL, -- 'affiliate_hotel', 'affiliate_ticket', 'whatsapp_lead', 'phone_call'
    target_url TEXT,
    user_agent TEXT,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 11. Trigger Otomatis Mengisi Kolom Geospasial (geom) dari Latitude & Longitude
CREATE OR REPLACE FUNCTION update_geom_column()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.latitude IS NOT NULL AND NEW.longitude IS NOT NULL THEN
        NEW.geom := ST_SetSRID(ST_MakePoint(NEW.longitude, NEW.latitude), 4326)::geography;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_cities_geom
BEFORE INSERT OR UPDATE OF latitude, longitude ON cities
FOR EACH ROW EXECUTE FUNCTION update_geom_column();

CREATE TRIGGER trg_places_geom
BEFORE INSERT OR UPDATE OF latitude, longitude ON places
FOR EACH ROW EXECUTE FUNCTION update_geom_column();

-- 12. Indeks Performa Tinggi untuk Pencarian 514 Kota & Geospasial
CREATE INDEX idx_cities_geom ON cities USING GIST (geom);
CREATE INDEX idx_places_geom ON places USING GIST (geom);
CREATE INDEX idx_places_city_category ON places (city_id, category_id, is_active);
CREATE INDEX idx_places_sponsored ON places (city_id, is_sponsored DESC, sponsored_tier DESC);
CREATE INDEX idx_news_city_published ON news_feeds (city_id, published_at DESC);
```

## 4. Alur Program & Arsitektur Sistem (Program Flow: TanStack Start + Rust)

### 4.1 Alur Otomasi Data (Rust Data Ingestion & Seeding Flow)
Alur ini menjelaskan bagaimana *binary* Rust mengisi data 514 kabupaten/kota di awal dan menjaganya tetap mutakhir:

1. **Tahap 1 — Injeksi Master Wilayah (`cargo run --bin seed_wilayah`):**
   * Binary CLI Rust mengambil data JSON dari [API Wilayah Indonesia (emsifa)](https://github.com/emsifa/api-wilayah-indonesia).
   * Memasukkan 38 baris ke tabel `provinces` dan 514 baris ke tabel `cities` beserta titik koordinat tengah dan pembuatan `slug` otomatis (contoh: `kota-bandung`, `kota-surabaya`, `kabupaten-sragen`).
2. **Tahap 2 — Ekstraksi POI Massal Se-Indonesia (`cargo run --release --bin seed_osm`):**
   * Mengunduh satu file `indonesia-latest.osm.pbf` dari [Geofabrik Indonesia](https://download.geofabrik.de/asia/indonesia.html).
   * Crate [osmpbf](https://crates.io/crates/osmpbf) membaca seluruh *node* dan *way* secara paralel di semua *core* CPU, menyaring tag wisata (`tourism=attraction|museum|viewpoint|theme_park`), kesehatan (`amenity=hospital|clinic|pharmacy`), dan transportasi (`public_transport=station`, `amenity=bus_station`).
   * Setiap titik dicocokkan ke `city_id` terdekat di PostGIS dan dimasukkan secara *batch* (`UNNEST`) ke tabel `places` dengan `ON CONFLICT (osm_id) DO UPDATE` dalam waktu kurang dari 3 menit.
3. **Tahap 3 — Worker Terjadwal di Dalam Server Rust (`tokio-cron-scheduler`):**
   * **Setiap 2 Jam:** Task background Rust mem-parsing puluhan RSS berita regional menggunakan [feed-rs](https://crates.io/crates/feed-rs/code), mencocokkan nama kota di judul berita ke `city_id`, lalu menyimpannya ke `news_feeds`.
   * **Cache Cuaca di Memori Rust:** Saat *endpoint* cuaca kota dipanggil, server Rust memeriksa *in-memory cache* (TTL 30 menit) sebelum memanggil [OpenWeatherMap API](https://www.google.com/search?q=https://openweathermap.org/api) agar hemat kuota *free tier*.

### 4.2 Alur Pengguna, SSR, & Programmatic SEO (TanStack Start ↔ Rust Axum)

```text
[Pengguna Membuka Website / Klik dari Pencarian Google]
       │
       ├──► Masuk via Beranda (/) ──► Cek Izin GPS Browser
       │       ├─► Diizinkan: Panggil Rust API (/api/v1/cities/nearest?lat=..&lng=..) ──► Redirect ke /kota/$citySlug
       │       └─► Ditolak: Tampilkan Search Combobox 514 Kota + Daftar Kota Populer (Tier-1)
       │
       └──► Masuk via Google SEO (/kota/kota-bandung/wisata atau /kota/kota-surabaya/rumah-sakit)
               │
               ▼
[Cloudflare CDN / Reverse Proxy Cache (Cek Header s-maxage=3600)]
       ├──► Cache HIT: Sajikan HTML statis instan (<20ms)
       └──► Cache MISS / Stale: Teruskan ke Server TanStack Start (SSR)
               │
               ▼
[TanStack Start Route Loader + Head Management]
       ├──► 1. Validasi URL Search Params (?lat, ?lng, ?radius, ?open24h) via Zod Schema
       ├──► 2. Panggil Backend Rust (GET /api/v1/kota/{citySlug}/{categorySlug})
       ├──► 3. Backend Rust mengeksekusi Query SQLx + PostGIS (Urutan: is_sponsored DESC, distance ASC)
       ├──► 4. TanStack Start merender Full HTML + Meta SEO + JSON-LD + Header Cache-Control ISR
       │
               ▼
[Interaksi Klien di Browser (Hydrated dengan TanStack Query + Leaflet/MapLibre)]
       ├──► Geser Peta / Klik Filter "Buka 24 Jam" ──► Update URL Search Params secara Type-Safe
       ├──► Klik "Cek Hotel / Tiket Terdekat" ──► POST /api/v1/clicks (Rust) ──► Redirect ke Link Affiliate
       └──► Klik "Koreksi Info / Klaim Bisnis" ──► POST /api/v1/submissions (Rust) ──► Masuk Antrean Admin
```

### 4.3 Alur Monetisasi UMKM & Klaim Bisnis (B2B Self-Serve Flow)
1. Pemilik usaha lokal membuka halaman tempat usahanya (yang sudah otomatis ter-*generate* dari data OpenStreetMap) lalu menekan tombol **"Klaim & Promosikan Usaha Ini"**.
2. Pemilik usaha mengisi nomor WhatsApp, jam buka terbaru, foto tempat, dan memilih paket:
   * **Klaim Gratis:** Pembaruan jam buka dan deskripsi (diverifikasi admin).
   * **Paket Featured UMKM (misal Rp50.000 – Rp150.000/bulan):** Mendapat lencana *Verified*, tombol pemesanan WhatsApp langsung, dan posisi teratas di halaman kota tersebut.
3. Admin memverifikasi bukti pembayaran melalui *endpoint* admin Rust, mengubah kolom `is_sponsored = true` dan `sponsored_until = NOW() + INTERVAL '30 days'`.
4. Halaman kota di TanStack Start otomatis menampilkan usaha tersebut di urutan paling atas.

## 5. Spesifikasi Teknis & Contoh Kode (Rust Backend + TanStack Start Frontend)

### 5.1 Konfigurasi Backend Rust (`Cargo.toml`)

```toml
[package]
name = "info-lokal-backend"
version = "0.1.0"
edition = "2021"

[dependencies]
axum = { version = "0.8", features = ["macros"] }
tokio = { version = "1.40", features = ["full"] }
tower-http = { version = "0.6", features = ["cors", "compression-gzip", "trace"] }
sqlx = { version = "0.8", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono", "rust_decimal"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
osmpbf = "0.3"
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
feed-rs = "2.0"
uuid = { version = "1.10", features = ["serde", "v4"] }
chrono = { version = "0.4", features = ["serde"] }
dotenvy = "0.15"
tracing = "0.1"
tracing-subscriber = "0.3"
```

### 5.2 Contoh Endpoint Geospasial di Rust (`Axum` + `SQLx` + `PostGIS`)

```rust
use axum::{extract::{Path, Query, State}, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Deserialize)]
pub struct DirectoryQuery {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_meters: Option<f64>,
    pub is_24_hours: Option<bool>,
}

#[derive(Serialize, FromRow)]
pub struct PlaceItem {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub is_24_hours: bool,
    pub is_sponsored: bool,
    pub distance_meters: Option<f64>,
    pub affiliate_hotel_url: Option<String>,
}

// Handler: Ambil daftar tempat per kota & kategori (mendukung filter GPS & 24 Jam)
pub async fn list_city_places(
    State(pool): State<PgPool>,
    Path((city_slug, category_slug)): Path<(String, String)>,
    Query(params): Query<DirectoryQuery>,
) -> Result<Json<Vec<PlaceItem>>, String> {
    let radius = params.radius_meters.unwrap_or(25_000.0); // Default 25 km
    let only_24h = params.is_24_hours.unwrap_or(false);

    let places = sqlx::query_as::<_, PlaceItem>(
        r#"
        SELECT 
            p.id, p.name, p.slug, p.address,
            p.latitude::FLOAT8 AS latitude,
            p.longitude::FLOAT8 AS longitude,
            p.is_24_hours, p.is_sponsored, p.affiliate_hotel_url,
            CASE 
                WHEN $1::FLOAT8 IS NOT NULL AND $2::FLOAT8 IS NOT NULL THEN
                    ST_Distance(p.geom, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography)
                ELSE NULL
            END AS distance_meters
        FROM places p
        JOIN cities c ON p.city_id = c.id
        JOIN categories cat ON p.category_id = cat.id
        WHERE c.slug = $3
          AND cat.slug = $4
          AND p.is_active = TRUE
          AND (NOT $5 OR p.is_24_hours = TRUE)
          AND (
              $1::FLOAT8 IS NULL OR $2::FLOAT8 IS NULL OR
              ST_DWithin(p.geom, ST_SetSRID(ST_MakePoint($1, $2), 4326)::geography, $6)
          )
        ORDER BY p.is_sponsored DESC, distance_meters ASC NULLS LAST, p.rating_avg DESC
        LIMIT 60
        "#,
    )
    .bind(params.lng)
    .bind(params.lat)
    .bind(city_slug)
    .bind(category_slug)
    .bind(only_24h)
    .bind(radius)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(Json(places))
}

pub fn create_router(pool: PgPool) -> Router {
    Router::new()
        .route("/api/v1/kota/{city_slug}/{category_slug}", get(list_city_places))
        .with_state(pool)
}
```

### 5.3 Contoh Rute Dinamis 514 Kota di TanStack Start (`src/routes/kota/$citySlug/$categorySlug/index.tsx`)

Kode berikut mendemonstrasikan bagaimana [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview) menangani **Type-Safe Search Params**, **SSR Loader ke API Rust**, **ISR via HTTP Cache Headers**, dan **Programmatic SEO (`head`)**:

```tsx
import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

// 1. Skema Type-Safe Search Params untuk Filter Peta & Direktori
const directorySearchSchema = z.object({
  lat: z.number().optional(),
  lng: z.number().optional(),
  radius: z.number().default(25000),
  open24h: z.boolean().optional(),
})

export const Route = createFileRoute('/kota/$citySlug/$categorySlug/')({
  validateSearch: (search) => directorySearchSchema.parse(search),

  loaderDeps: ({ search: { lat, lng, radius, open24h } }) => ({
    lat, lng, radius, open24h,
  }),

  // 2. SSR Loader memanggil Backend Rust (Axum)
  loader: async ({ params: { citySlug, categorySlug }, deps }) => {
    const query = new URLSearchParams({
      radius_meters: String(deps.radius),
      ...(deps.lat && deps.lng ? { lat: String(deps.lat), lng: String(deps.lng) } : {}),
      ...(deps.open24h ? { is_24_hours: 'true' } : {}),
    })

    const res = await fetch(
      `${process.env.RUST_API_URL}/api/v1/kota/${citySlug}/${categorySlug}?${query}`
    )
    if (!res.ok) throw new Error('Gagal memuat direktori kota dari server Rust')
    const places = await res.json()
    return { citySlug, categorySlug, places }
  },

  // 3. ISR Caching via Standard HTTP Header (Bisa di-cache Cloudflare / CDN)
  headers: () => ({
    'Cache-Control': 'public, max-age=300, s-maxage=3600, stale-while-revalidate=86400',
  }),

  // 4. Programmatic SEO Otomatis untuk 514 Kabupaten/Kota
  head: ({ loaderData }) => {
    if (!loaderData) return {}
    const { citySlug, categorySlug, places } = loaderData
    const formattedCity = citySlug.replace(/-/g, ' ').toUpperCase()
    const formattedCat = categorySlug.replace(/-/g, ' ').toUpperCase()
    const title = `${places.length}+ ${formattedCat} Terdekat di ${formattedCity} beserta Peta & Rute`

    return {
      meta: [
        { title },
        {
          name: 'description',
          content: `Daftar lengkap ${formattedCat} di ${formattedCity}, jam operasional, nomor darurat, dan rute navigasi terkini.`,
        },
      ],
    }
  },

  component: CityDirectoryComponent,
})

function CityDirectoryComponent() {
  const { places } = Route.useLoaderData()
  const search = Route.useSearch()
  const navigate = Route.useNavigate()

  return (
    <section className="max-w-6xl mx-auto p-4">
      <div className="mb-4 flex items-center gap-3">
        <button
          onClick={() =>
            navigate({
              search: (prev) => ({ ...prev, open24h: !prev.open24h }),
            })
          }
          className="px-4 py-2 rounded bg-emerald-600 text-white font-medium"
        >
          {search.open24h ? 'Tampilkan Semua Jam Buka' : 'Filter Buka 24 Jam (IGD / Apotek)'}
        </button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {places.map((place: any) => (
          <article key={place.id} className="border rounded-lg p-4 shadow-sm">
            {place.is_sponsored && (
              <span className="text-xs bg-amber-100 text-amber-800 px-2 py-1 rounded font-semibold">
                Mitra Terverifikasi
              </span>
            )}
            <h2 className="text-lg font-bold mt-1">{place.name}</h2>
            <p className="text-sm text-gray-600">{place.address ?? 'Alamat mengikuti titik koordinat peta'}</p>
            {place.affiliate_hotel_url && (
              <a
                href={place.affiliate_hotel_url}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-3 inline-block text-sm text-blue-600 font-semibold"
              >
                Cek Hotel & Tiket Wisata Terdekat →
              </a>
            )}
          </article>
        ))}
      </div>
    </section>
  )
}
```

## 6. Step-by-Step Development Plan (Panduan Eksekusi Bertahap: Rust + TanStack Start)

### Fase 1: Persiapan Database PostGIS & Backend Rust (Hari 1–4)
1. **Siapkan Database PostgreSQL + PostGIS:**
   * Buat database di [Supabase](https://www.google.com/search?q=https://supabase.com/pricing) atau instal PostgreSQL + PostGIS di VPS Linux Anda.
   * Instal CLI migrasi Rust: `cargo install sqlx-cli --no-default-features --features rustls,postgres`.
   * Jalankan `sqlx migrate run` untuk mengeksekusi seluruh skema tabel, *trigger* `geom`, dan indeks `GIST` pada **Bagian 3**.
2. **Inisialisasi Workspace Rust (`info-lokal-backend`):**
   * Buat struktur proyek Rust yang memisahkan server API (`src/main.rs`) dan *binary* pengimpor data (`src/bin/seed_wilayah.rs` & `src/bin/seed_osm.rs`).
   * Siapkan koneksi `PgPool` menggunakan [sqlx](https://crates.io/crates/sqlx) dan *middleware* CORS serta kompresi Gzip di [axum](https://crates.io/crates/axum).

### Fase 2: Seeding Otomatis 514 Kota Se-Indonesia dengan Rust (Hari 5–7)
1. **Impor 38 Provinsi & 514 Kabupaten/Kota:**
   * Jalankan `cargo run --release --bin seed_wilayah` untuk mengambil data JSON dari [API Wilayah Indonesia (emsifa)](https://github.com/emsifa/api-wilayah-indonesia) dan mengisi tabel `provinces` serta `cities`.
2. **Seeding Nomor Darurat Nasional:**
   * Masukkan data kontak darurat nasional (`112`, `119`, `110`, `113`, `115`) ke tabel `emergency_contacts`.
3. **Ekstraksi Cepat File PBF Seluruh Indonesia (`osmpbf`):**
   * Unduh `indonesia-latest.osm.pbf` dari [Geofabrik Indonesia](https://download.geofabrik.de/asia/indonesia.html).
   * Jalankan `cargo run --release --bin seed_osm` yang memproses seluruh titik wisata, rumah sakit, klinik, apotek, terminal, dan stasiun di 514 kabupaten/kota ke tabel `places`.

### Fase 3: Pengembangan Front-End TanStack Start & Programmatic SEO (Minggu 2)
1. **Inisialisasi Proyek TanStack Start:**
   * Buat proyek menggunakan [TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview), Tailwind CSS, Zod, dan TanStack Query.
2. **Bangun File-Based Routing untuk 514 Kota:**
   * `src/routes/index.tsx` — Beranda Nasional + Deteksi Lokasi GPS + Pencarian 514 Kota.
   * `src/routes/kota/$citySlug/index.tsx` — Dashboard Kota (Cuaca, Nomor Darurat, Top Wisata, IGD 24 Jam, Berita Daerah).
   * `src/routes/kota/$citySlug/$categorySlug/index.tsx` — Halaman Kategori dengan Peta Interaktif & Filter Parameter URL *Type-Safe* ([TanStack Router Search Params](https://tanstack.com/router/latest/docs/guide/search-params)).
   * `src/routes/kota/$citySlug/$categorySlug/$placeSlug.tsx` — Halaman Detail Tempat + Tombol Rute Navigasi & CTA Afiliasi.
3. **Konfigurasi SEO, JSON-LD, & Sitemap XML:**
   * Terapkan fungsi `head()` pada setiap rute dinamis ([TanStack Start SEO](https://tanstack.com/start/latest/docs/framework/react/guide/seo)) serta *header* `Cache-Control` untuk [Incremental Static Regeneration (ISR)](https://tanstack.com/start/latest/docs/framework/react/guide/isr).
   * Buat rute server `/sitemap.xml` yang men-*generate* daftar URL 514 kota × kategori dari *endpoint* Rust untuk didaftarkan ke Google Search Console.

### Fase 4: Peta Mandiri (Martin Tile Server), Cuaca, & Berita RSS (Minggu 3)
1. **Integrasi Peta Interaktif & Martin Tile Server:**
   * Pasang komponen peta ([Leaflet.js](https://www.google.com/search?q=https://leafletjs.com/) atau MapLibre GL) di sisi klien dengan atribusi `© OpenStreetMap contributors`.
   * Untuk menghindari batas [OSM Tile Usage Policy](https://www.google.com/search?q=https://operations.osmfoundation.org/policies/tiles/), jalankan [Martin Tile Server](https://github.com/maplibre/martin) berbasis Rust di server Anda untuk menyajikan *vector tiles* secara mandiri.
2. **Worker Berita RSS & Cuaca di Rust:**
   * Aktifkan *background task* di server Rust setiap 2 jam menggunakan crate [feed-rs](https://crates.io/crates/feed-rs/code) untuk memperbarui tabel `news_feeds` dan menyediakan *endpoint* cuaca ter-*cache* dari [OpenWeatherMap API](https://www.google.com/search?q=https://openweathermap.org/api).
3. **Formulir Crowdsourcing & Klaim Bisnis:**
   * Hubungkan form *"Koreksi Data / Klaim Usaha"* di TanStack Start ke *endpoint* `POST /api/v1/submissions` di Rust.

### Fase 5: Monetisasi, Kurasi Kota Tier-1, & Deployment (Minggu 4)
1. **Integrasi Generator Link Afiliasi & Tracking Klik:**
   * Daftar ke [Traveloka Affiliate Program](https://www.google.com/search?q=https://www.traveloka.com/id-id/p/affiliate) dan [Blibli Tiket Affiliate](https://www.google.com/search?q=https://www.tiket.com/id-id/explore/blibli-tiket-affiliate).
   * Arahkan setiap klik tombol afiliasi dan panggilan WhatsApp UMKM melalui *endpoint* pelacak di Rust (`POST /api/v1/clicks`) agar tercatat di tabel `monetization_clicks`.
2. **Kurasi Mendalam 3–5 Kota Tier-1 Pertama:**
   * Lengkapi data harga tiket wisata utama, jadwal *event* lokal, koridor transportasi publik, dan nomor darurat daerah pada kota fokus awal (misalnya Bandung, Surabaya, Malang, Yogyakarta, dan Solo/Sragen).
3. **Deployment Hemat Biaya & Pengajuan AdSense:**
   * Deploy *binary* Rust (`Axum`) dan server SSR `TanStack Start` di VPS atau layanan kontainer di balik **Cloudflare CDN (Free Plan)** agar halaman 514 kota di-*cache* di *edge server*.
   * Pantau indeksasi di Google Search Console dan daftarkan domain ke [Google AdSense](https://support.google.com/adsense/answer/9234653?hl=en).