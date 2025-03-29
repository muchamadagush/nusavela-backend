# DESIGN.md - NusaVela (`nusavela.com`) Design System & UI Specification

Dokumen ini adalah panduan desain utama (*Single Source of Truth*) untuk pengembangan antarmuka web ([TanStack Start](https://tanstack.com/start/latest/docs/framework/react/overview)) dan aplikasi mobile ([Flutter](https://flutter.dev/)) **NusaVela (`nusavela.com`)** - *Layar Penjelajah Pesona & Informasi Nusantara (514 Kabupaten/Kota)*.

## 1. Tipe & Filosofi Desain (Design Archetype)

**Tipe Desain Utama:** **Archipelago Editorial Luxury + Spatial Bento Utility**
*(Perpaduan kemewahan editorial majalah perjalanan kelas atas dengan kejelasan fungsional peta geospasial dan direktori layanan publik).*

### 4 Pilar Karakter Visual NusaVela
1. **Quiet Luxury & Warm Editorial (Kesan Megah & Expensive):**
   * Menghindari warna latar putih pucat yang kaku atau warna neon yang murahan. Menggunakan latar *Warm Alabaster / Pearl Sand* (`#FAF8F5`) dengan aksen *Deep Vela Navy* (`#0B1F3A`), *Emerald Nusantara* (`#0D5C46`), dan *Swarna Gold* (`#C89B3C`).
   * Menggunakan tipografi judul *Editorial Serif* pada bagian *Hero* kota untuk menonjolkan keagungan setiap daerah di Indonesia (seperti sampul majalah *Condé Nast Traveler* atau *Aman Resorts*).
2. **Spatial Map-First Experience (Berpusat pada Peta & Jarak):**
   * Pada layar Desktop (`>= 1024px`), halaman kategori kota menggunakan tata letak **Split-Screen 55% Daftar Direktori : 45% Peta Interaktif Sticky** ([Leaflet.js](https://leafletjs.com/) / MapLibre).
   * Pada layar Mobile (`< 768px`), menggunakan **Interactive Map + Draggable Bottom Sheet** sehingga pengguna dapat menggeser peta dengan satu jempol.
3. **Bento Grid City Dashboard (Modular & Terstruktur):**
   * Halaman utama setiap kota (`/kota/$citySlug`) disusun dalam *Bento Grid* asimetris yang menampilkan Cuaca Saat Ini, Tombol Cepat Darurat 112/119, Top 4 Destinasi Wisata, IGD RS 24 Jam Terdekat, dan Berita Daerah Terkini dalam satu pandangan tanpa perlu *scrolling* panjang.
4. **Strict Separation of Commercial vs. Emergency UI (Etika Visual):**
   * Elemen komersial (*Mitra Terverifikasi / Sponsored UMKM*) menggunakan aksen *Swarna Gold* (`#C89B3C`).
   * Elemen keselamatan jiwa (**Nomor Darurat 112/119** & **IGD 24 Jam**) menggunakan warna *Siaga Crimson* (`#DC2626`) dan dilarang keras berdampingan dengan *banner* iklan *pop-up*.

## 2. Color Palette & Token Warna (WCAG AA/AAA Compliant)

### 2.1 Core Brand & Semantic Palette (Light & Dark Mode)

| Token / Peran | Nama Warna | Hex (Light Mode) | Hex (Dark Mode) | Penggunaan Utama di UI & Peta |
| :--- | :--- | :--- | :--- | :--- |
| `--bg-canvas` | **Pearl Sand / Abyssal** | `#FAF8F5` | `#07111E` | Latar belakang utama halaman (hangat, tidak menyilaukan mata). |
| `--bg-surface` | **Pure Ivory / Deep Slate** | `#FFFFFF` | `#0F1E32` | Kartu direktori tempat wisata, modal klaim bisnis, dan panel navigasi. |
| `--bg-subtle` | **Limestone / Midnight** | `#F2EFE9` | `#162942` | Latar bagian filter, *skeleton loading*, dan *chip* kategori pasif. |
| `--brand-primary` | **Deep Vela Navy** | `#0B1F3A` | `#38BDF8` | Header utama, tombol navigasi primer, dan identitas samudra Nusantara. |
| `--brand-secondary` | **Emerald Nusantara** | `#0D5C46` | `#34D399` | Kategori Wisata & Alam, lencana status "Buka Sekarang", aksen tropis. |
| `--accent-luxury` | **Swarna Gold (Kencana)** | `#C89B3C` | `#FBBF24` | **Khusus Monetisasi:** Lencana *NusaVela Verified*, *Pin Peta Emas*, dan *border* kartu *Spotlight UMKM*. |
| `--accent-gold-bg` | **Champagne Surface** | `#FDF8EC` | `#292111` | Latar belakang halus untuk kartu *Sponsored / Featured Listing*. |
| `--action-affiliate` | **Royal Ocean Blue** | `#1D4ED8` | `#60A5FA` | Tombol CTA berkonversi tinggi: *"Cek Hotel & Tiket Terdekat"*. |
| `--alert-emergency` | **Siaga Crimson (SOS)** | `#DC2626` | `#F87171` | **Khusus Darurat:** Tombol Panggilan 112/119, lencana **IGD 24 Jam**, & peringatan cuaca ekstrem. |
| `--text-primary` | **Obsidian Ink** | `#111827` | `#F9FAFB` | Judul tempat, isi artikel, dan teks utama (rasio kontras > 14:1). |
| `--text-secondary` | **Warm Charcoal** | `#4B5563` | `#9CA3AF` | Alamat, jam operasional, ringkasan berita RSS (rasio kontras > 5.5:1). |
| `--border-hairline` | **Sandstone Border** | `#E6E2D8` | `#1E334D` | Garis tepi kartu `1px solid`, pembatas tabel, dan *divider* halus. |

### 2.2 Warna Pin & Marker Peta Interaktif (Category Map Pins)
Agar peta di 514 kota mudah dipindai secara visual dalam sekali lihat:
* **Pin Wisata & Budaya:** *Emerald Nusantara* (`#0D5C46`) dengan ikon kompas/pohon.
* **Pin Rumah Sakit & IGD 24 Jam:** *Siaga Crimson* (`#DC2626`) dengan ikon palang medis putih.
* **Pin Kuliner & UMKM:** *Terracotta Spice* (`#C2410C`) dengan ikon garpu/sendok.
* **Pin Transportasi (Stasiun/Terminal/Bandara):** *Royal Ocean Blue* (`#1D4ED8`) dengan ikon kereta/bus.
* **Pin Mitra Spotlight (Berbayar B2B):** *Swarna Gold* (`#C89B3C`) berukuran `1.25x` lebih besar dengan efek cincin berkilau (*pulse ring*) dan lencana bintang.

## 3. Tipografi (Typography Scale & Font Pairing)

Menggabungkan keanggunan *Editorial Serif* dengan keterbacaan *UI Sans-Serif* karya desainer Indonesia:

* **Font Display / Hero Kota (`--font-display`):** `'Playfair Display'` atau `'Instrument Serif'` (Digunakan khusus untuk nama kota di *Hero Banner*, contoh: *"Menjelajahi Kota Bandung"* atau *"Kabupaten Sragen"*).
* **Font UI & Body Utama (`--font-sans`):** `'Plus Jakarta Sans'`, `-apple-system`, `sans-serif` (Digunakan untuk seluruh judul kartu direktori, filter, navigasi, dan berita).
* **Font Angka, Jarak, & Nomor Darurat (`--font-mono`):** `'JetBrains Mono'`, `monospace` (Digunakan untuk indikator jarak `1.4 km`, harga tiket `Rp25.000`, koordinat GPS, dan nomor darurat `112` / `119`).

| Elemen Tipografi | Ukuran Desktop | Ukuran Mobile | Ketebalan (Weight) | Line Height |
| :--- | :--- | :--- | :--- | :--- |
| **City Hero Title (H1)** | `2.75rem` (44px) | `2.0rem` (32px) | `700` (Bold Serif) | `1.15` |
| **Section / Category Heading (H2)** | `1.75rem` (28px) | `1.375rem` (22px) | `700` (Plus Jakarta) | `1.25` |
| **Place Card Title (H3)** | `1.125rem` (18px) | `1.0625rem` (17px) | `600` (SemiBold) | `1.35` |
| **Body / Deskripsi Lokasi** | `1.0rem` (16px) | `0.9375rem` (15px) | `400` (Regular) | `1.60` |
| **Metadata / Jam Buka / Badge** | `0.8125rem` (13px) | `0.75rem` (12px) | `500` (Medium) | `1.40` |

## 4. Spesifikasi Komponen Kunci & Tailwind v4 CSS Variables

```css
:root {
  --bg-canvas: #FAF8F5;
  --bg-surface: #FFFFFF;
  --bg-subtle: #F2EFE9;
  --brand-primary: #0B1F3A;
  --brand-secondary: #0D5C46;
  --accent-luxury: #C89B3C;
  --accent-gold-bg: #FDF8EC;
  --action-affiliate: #1D4ED8;
  --alert-emergency: #DC2626;
  --text-primary: #111827;
  --text-secondary: #4B5563;
  --border-hairline: #E6E2D8;
  --radius-card: 16px;
  --radius-pill: 9999px;
  --shadow-luxury: 0 10px 30px -10px rgba(11, 31, 58, 0.07);
}
```

### Aturan Komponen Kartu Direktori (Place Card)
1. **Kartu Reguler (Data Otomatis OSM):** Latar `--bg-surface`, *border* `1px solid var(--border-hairline)`, sudut `16px`, menampilkan kategori, nama tempat, jarak (`font-mono`), alamat singkat, status buka/tutup, serta tombol *"Rute"* dan *"Cek Hotel Terdekat"*.
2. **Kartu Mitra Berbayar (NusaVela Spotlight Card):** Latar `--accent-gold-bg` (`#FDF8EC`), *border* `1.5px solid var(--accent-luxury)` (`#C89B3C`), lencana emas **"Rekomendasi Mitra"** di sudut kanan atas, serta tombol hijau WhatsApp langsung.
3. **Bar Darurat Sticky (SOS Bar):** Pada halaman kategori `rumah-sakit` dan `layanan-publik`, tampilkan *bar* bagian atas berlatar `#FEF2F2` dengan teks & tombol `#DC2626` untuk panggilan 1-klik ke `112` (Darurat Nasional) dan `119` (Ambulans).
