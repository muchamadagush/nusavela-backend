# NusaVela Backend

Backend Rust Axum 0.8 + SQLx 0.8 + PostgreSQL/PostGIS untuk NusaVela. Kontrak inti cocok dengan paket frontend sebelumnya: array kota/kategori/tempat, JSON number untuk koordinat, dan rute `/api/v1/kota/{city}/{category}/{place}`.

Repository ini memetakan direktori `backend/` pada blueprint ke root proyek. Migrasi berada di `migrations/`. Tidak ada penyimpanan SQLite. Berkas database/data nasional, password produksi, API key, token admin, dan file PBF Indonesia tidak disertakan.

## Mulai dengan Docker

```sh
cp .env.example .env
# Isi ADMIN_JWT_SECRET dengan secret acak minimal 32 byte untuk mengaktifkan admin.
# Password lokal di contoh hanya untuk pengembangan. Isi POSTGRES_PASSWORD bila memakai Compose.
docker compose up --build -d
curl http://localhost:8080/healthz
curl http://localhost:8080/readyz
curl http://localhost:8080/api/v1/categories
```

Server menjalankan migrasi tertanam sebelum menerima request. Gunakan database kosong untuk instalasi pertama; jangan mengubah migrasi yang sudah diterapkan. Untuk produksi, pisahkan role migrasi yang dapat memasang ekstensi dengan role API berprivilege minimum. Contoh Compose mengikat port ke localhost untuk diakses reverse proxy milik Anda.

## Jalankan secara native

Gunakan Rust stable beserta `cargo`, `clippy`, `rustfmt`, PostgreSQL 16+ dan PostGIS 3.4+.

```sh
cp .env.example .env
cargo run --locked --bin nusavela-backend
```

Konfigurasi utama:

| Variabel | Fungsi |
| --- | --- |
| `DATABASE_URL` | Koneksi PostgreSQL/PostGIS. |
| `DATABASE_MAX_CONNECTIONS` | Default 10, rentang 1–100. RSS memerlukan minimal 2. |
| `BIND_ADDR` | Default `0.0.0.0:8080`. |
| `CORS_ORIGINS` | Daftar origin tepat, dipisahkan koma, tanpa path/wildcard. |
| `ADMIN_JWT_SECRET` | Secret minimal 32 byte; kosong membuat seluruh admin ditolak. |
| `ADMIN_JWT_ISSUER`, `ADMIN_JWT_AUDIENCE` | Default `nusavela-admin`, `nusavela-api`. |
| `OPENWEATHER_API_KEY` | Kosong membuat endpoint cuaca mengembalikan 503. |
| `RSS_ENABLED` | Default false. Isi feed tepercaya di `provinces.rss_news_urls`, lalu set true. |

## Isi data wilayah dengan benar

`seed_wilayah` membutuhkan snapshot JSON terversi yang berisi **kode, nama, koordinat dan timezone**. Data emsifa saja tidak menyediakan seluruh kebutuhan ini; versi lamanya juga dapat berbeda cakupan provinsi. Importer menolak jumlah yang tidak sesuai dan membungkus seluruh impor dalam satu transaksi.

```sh
cargo run --locked --bin seed_wilayah -- --file /path/wilayah-terverifikasi.json
# Opsional: cocokkan kode/nama dengan endpoint emsifa-compatible yang versinya sama.
cargo run --locked --bin seed_wilayah -- --file /path/wilayah-terverifikasi.json \
  --verify-emsifa-base https://www.emsifa.com/api-wilayah-indonesia/api
```

Struktur data mengikuti `tests/fixtures/wilayah.test.json`. **Fixture tersebut sintetis dan bukan data Indonesia**; default importer mengharuskan 38 provinsi/514 kabupaten-kota. Flag jumlah hanya dipakai untuk dataset uji atau cakupan yang sengaja disepakati. Database kosong tidak berarti 514 kota telah terisi.

## Impor OpenStreetMap

Unduh PBF dari sumber resmi Geofabrik dan periksa checksum yang diterbitkan penyedia. File besar tidak diunduh otomatis saat server berjalan.

```sh
cargo run --release --locked --bin seed_osm -- \
  --file /path/indonesia-latest.osm.pbf --batch-size 5000 --max-city-distance 50000
```

Parser paralel `osmpbf::par_map_reduce` mengirim node dan kandidat POI ke antrean terbatas. Node dipakai untuk menyelesaikan geometri way setelah parsing. Penulisan node memakai UNNEST; kandidat memakai batch JSONB. Identitas `(osm_type,osm_id)` mencegah benturan node/way. Proses memakai advisory lock, batch keyset, dan upsert idempotent. Kolom kurasi, sponsor, dan afiliasi tidak ditimpa impor.

Isi `cities.boundary` dengan MultiPolygon administratif SRID 4326 untuk penempatan kota berbasis batas wilayah. Jika tidak tersedia, importer memakai pusat kota terdekat dalam radius yang diberikan dan menandainya `nearest_centroid`. Way memakai rata-rata koordinat referensi node, bukan centroid area presisi. Way dengan referensi tidak lengkap dan POI tanpa kota dalam radius dilewati. Relation belum diimpor. Staging disimpan untuk pemeriksaan dan dibangun ulang pada impor berikutnya; kelanjutan byte-offset/checkpoint belum tersedia. Tidak ada janji impor nasional selesai 3 menit.

## Admin dan moderasi

Tidak ada endpoint login/password bawaan atau token admin statis. Terbitkan JWT lewat CLI tepercaya dengan secret yang sama seperti server:

```sh
cargo run --locked --bin mint_admin_token -- --subject operator-anda --seconds 900
```

Simpan token di lingkungan operator, kirim sebagai `Authorization: Bearer ...`. Jangan masukkan secret ke frontend. JWT memvalidasi HS256, signature, expiry, not-before, issuer, audience, dan role admin. Token berlaku paling lama satu jam dari CLI; rotasi secret membatalkan token lama.

Alur koreksi/klaim: kontribusi publik masuk `pending`; admin menyetujui dengan patch eksplisit atau menolak dengan alasan. Peninjauan memakai row lock dan audit transaksi, sehingga request yang sudah ditinjau tidak diproses ulang. Klaim bukan pemberian kepemilikan akun atau bukti pembayaran otomatis. Untuk `new_place`/`new_event`, admin membuat entitas terkurasi melalui endpoint terkait, lalu mereview kontribusi dengan `created_entity_id` dari kota yang sama.

Sponsor diaktifkan terpisah melalui endpoint sponsorship, dengan referensi pembayaran yang telah diperiksa operator, tier, dan expiry. Tidak ada integrasi payment gateway atau verifikasi pembayaran otomatis.

## Peta Martin

Migrasi menyertakan fungsi `place_tiles(z,x,y)` yang menghasilkan layer MVT `places` dari listing aktif dan sponsor efektif. Jalankan opsi Martin dengan `docker compose --profile maps up -d`; endpoint fungsi yang diharapkan `/place_tiles/{z}/{x}/{y}` pada port 3001. Periksa katalog Martin sebelum menghubungkan klien.

Tile ini adalah vector MVT. Frontend sebelumnya memakai Leaflet raster dan **tidak boleh** menerima URL MVT sebagai `VITE_MAP_TILE_URL`. Gunakan MapLibre + style vector atau adapter/provider raster. Penyedia tile, style, font dan atribusi OSM tetap harus dikonfigurasi. Container Martin/Docker belum dijalankan dalam lingkungan verifikasi paket ini.

## Pengujian

```sh
cargo fmt --check
cargo check --locked --all-targets
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

Uji integrasi membutuhkan **database uji terpisah yang boleh dikosongkan**, bukan database produksi:

```sh
export TEST_DATABASE_URL=postgres://postgres:postgres@localhost:5432/nusavela_test
cargo test --locked --test api_database -- --ignored --nocapture
cargo test --locked --test importers -- --ignored --nocapture
```

Uji API mengosongkan tabel aplikasi untuk membuat fixture terkontrol. Uji importers menjalankan kedua binary CLI dua kali, memeriksa benturan ID node/way, perlindungan kurasi/sponsor, dan penolakan cakupan nasional palsu. Delapan unit test berjalan tanpa database. Workflow CI menyediakan PostgreSQL/PostGIS native untuk menjalankan seluruh pemeriksaan.

Untuk uji SSR lintas paket setelah fixture importer tersedia dan API aktif di localhost:8080:

```sh
# Bangun frontend di repo sebelah terlebih dahulu.
node scripts/frontend-smoke.mjs ../nusavela-frontend/dist/server/server.js
```

Lihat [docs/API.md](docs/API.md) dan [docs/VERIFICATION.md](docs/VERIFICATION.md) untuk kontrak serta batas verifikasi.
