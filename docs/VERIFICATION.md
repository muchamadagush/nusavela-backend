# Hasil verifikasi backend NusaVela

Paket dibuat 31 Oktober 2026. Dokumen AGENTS, SKILL, DESIGN dan PRD lampiran digunakan sebagai referensi. Struktur monorepo diadaptasi menjadi repository backend terpisah sesuai permintaan pengguna. Tidak ada push remote: pengguna menerima pengiriman ZIP setelah akses GitHub sebelumnya gagal.

## Cakupan implementasi

- Server Axum 0.8, startup migration SQLx, CORS origin eksplisit, kompresi, tracing, JSON error terstruktur, timeout request/query, pembatasan body dan write, SIGINT/SIGTERM shutdown.
- Migrasi PostgreSQL/PostGIS: provinsi, kota, kategori, tempat, kontak darurat, event, berita, kontribusi, klik, audit, limiter, staging OSM, dan fungsi tile Martin. Geometry dihitung sebagai generated stored column; pemeriksaan koordinat dan relasi dipaksakan database.
- Pencarian kota, nearest-city, daftar/detail tempat, filter jarak/24 jam/IGD, rating/paginasi dan sponsor efektif dengan stable ID.
- Kontribusi pending, validasi hubungan city/place, moderasi admin transaksi/row lock, kurasi tempat/event/kontak, sponsor dan affiliate, agregat klik 30 hari.
- JWT admin HS256 memvalidasi signature, role, subject, issuer, audience, expiry, not-before. Tidak ada token admin default atau credential frontend.
- Cuaca OpenWeather dengan TTL 30 menit dan atribusi, worker RSS dua jam, ringkasan maksimal 160 karakter, batas ukuran feed, DNS/IP validation, DNS pinning, dan redirect dinonaktifkan.
- Seeder wilayah terversi dengan coverage validation serta opsi verifikasi emsifa; PBF parallel ingestion dengan bounded channel, batch, way-node resolution, identitas tipe/ID, assignment boundary/fallback, dan perlindungan kurasi/sponsor saat upsert.
- Dockerfile, Compose PostGIS/API/Martin opsional, CI dengan PostgreSQL/PostGIS native, unit/integration tests dan live SSR smoke script.

## Pemeriksaan yang dijalankan

| Pemeriksaan | Hasil / batas |
| --- | --- |
| `cargo check --locked --all-targets` | Lulus pada toolchain Rust stable 1.99.1 yang dipasang dalam runtime kerja. |
| `cargo clippy --locked --all-targets -- -D warnings` | Lulus. |
| `cargo test --locked` | Delapan unit test lulus; integration test memerlukan flag `--ignored` dan database uji. |
| Migrasi dan integration API | Lulus dengan PostgreSQL WASM PGlite + ekstensi PostGIS; bukan mock query. MVT encoding dikecualikan secara eksplisit. |
| Importer binary wilayah dan PBF | Lulus: dijalankan dua kali, identitas node/way tetap terpisah, kurasi dan sponsor dipertahankan, dataset uji kecil ditolak sebagai data nasional default. |
| Frontend SSR ↔ API Rust ↔ PostGIS | Empat rute lulus: beranda, kota, kategori dan detail; menggunakan paket frontend sebelumnya serta fixture database sintetis. |
| Fungsi MVT | Migrasi fungsi dibuat, tetapi encoding belum lulus verifikasi lokal: PostGIS WASM tidak dikompilasi dengan protobuf-c. CI native menjalankan test ini tanpa pengecualian. |
| Container Docker/Martin dan CI GitHub | File disiapkan; belum dijalankan/dipush. |
| Cuaca/RSS publisher langsung | Belum diuji memakai kredensial cuaca dan feed produksi. Unit test parsing/truncation/SSRF berjalan. |
| Beban nasional/performa | Belum diuji; tidak mengklaim 38/514 data sudah terisi atau impor selesai 1–3 menit. |

Test database di runtime ini memakai `@electric-sql/pglite` 0.5.14, `@electric-sql/pglite-postgis` 0.2.14 dan `@electric-sql/pglite-socket` 0.2.17. Adapter verifikasi memberi namespace prepared statement per koneksi karena PGlite memakai satu sesi dasar; SQL dan hasil query tidak diubah. Ini tidak menguji isolasi multi-session PostgreSQL native, deadlock, concurrency atau load. Jalankan workflow CI native dan integration suite pada staging sebelum penggunaan produksi. Adapter pengujian ini tidak menjadi bagian runtime produksi backend.

Fixture dan server verifikasi tidak memuat data pribadi/produksi. Nomor darurat dari PRD hanya dimasukkan sebagai draf tidak aktif; endpoint publik hanya menampilkan kontak aktif yang ditinjau operator dengan sumber dan waktu verifikasi.

## Perbedaan penting dari contoh PRD

1. Tidak memakai `osm_id` saja sebagai identitas; namespace node/way/relation berbeda.
2. Sponsor kedaluwarsa tidak mendapat ranking/badge aktif. Sponsor tidak otomatis menjadi bukti verifikasi atau kepemilikan akun.
3. Latitude/longitude finite, berpasangan, dan longitude diikat pertama dalam ST_MakePoint. Koordinat nol tetap sah; kasus koordinat asimetris juga diuji.
4. Dynamic `query_as` tidak disebut sebagai validasi SQL saat compile; SQL diuji terhadap database.
5. RSS memakai batas PRD yang lebih ketat: 160 karakter. Penempatan kota ambigu tidak ditebak dari substring; berita tanpa city assignment disajikan pada scope provinsi.
6. Importer tidak mengarang koordinat atau memperlakukan dataset wilayah lama sebagai cakupan mutakhir. Kode/nama/koordinat harus digabung dan diperiksa operator dari sumber yang sesuai.
7. Tile MVT tidak dipasang ke komponen Leaflet raster frontend. MapLibre/style atau adapter raster masih diperlukan.
8. Tidak mengaktifkan daftar kontak darurat atau integrasi pembayaran/affiliate sebagai fakta terverifikasi tanpa tindakan operator.

## Pekerjaan yang tetap memerlukan data/lingkungan pengguna

- Snapshot wilayah nasional terbaru, koordinat dan batas administratif; file PBF dan pengecekan checksum.
- Verifikasi kontak/IGD, harga tiket, jadwal event/transport lokal dan klaim usaha.
- OpenWeather API key, daftar feed yang berizin, link affiliate resmi dan bukti pembayaran sponsor.
- Konfigurasi peta, frontend MapLibre bila memakai vector tiles, domain, TLS/CDN dan reverse proxy/rate limit GET serta IP asli.
- Benchmark impor dan query geospasial dengan dataset penuh, pengujian concurrency PostgreSQL native, deployment dan backup/restore.

Flutter, AI itinerary, payment gateway, login/ownership akun bisnis, UI admin, dan importer relation OSM tidak diimplementasikan dalam paket backend ini. Endpoint admin menerima tindakan operator tepercaya; kontribusi publik tidak otomatis menjadi data terverifikasi.

## Referensi teknis

- https://docs.rs/axum/0.8.8/axum/
- https://docs.rs/sqlx/0.8.6/sqlx/
- API `ElementReader::par_map_reduce` diverifikasi dari source crate `osmpbf` 0.3.8 yang dipakai build.
- https://maplibre.org/martin/config-file/
