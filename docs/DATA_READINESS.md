# Intake data nasional dan kurasi

Sumber PRD: master kode/nama dari API Wilayah Indonesia (emsifa), POI OSM dari Geofabrik Indonesia. Dataset wilayah perlu snapshot terversi dengan 38 provinsi/514 kabupaten-kota, pusat koordinat dan timezone yang benar. API kode/nama saja tidak memenuhi kebutuhan koordinat; versi administratif harus cocok. Fixture sintetis bukan data nasional.

Sebelum akses database: `cargo run --locked --bin seed_wilayah -- --file /path/wilayah.json --check-only`. Laporan JSON menyebut jumlah/source/version dan imported=false. Opsional --verify-emsifa-base memeriksa kode/nama terhadap sumber versi yang sama; flag ini membutuhkan jaringan. Validasi skema/count bukan verifikasi lapangan atau bukti lisensi.

Setelah snapshot, lisensi dan koordinat ditinjau operator, jalankan tanpa --check-only untuk impor atomik. Catat SHA-256 berkas dan sumber unduhan dalam arsip rilis operasional. Unduh indonesia-latest.osm.pbf dari Geofabrik dengan checksum resmi dan verifikasi sebelum seed_osm. Jalankan uji importer pada database disposable, kemudian impor ke staging; jangan gunakan flag jumlah fixture untuk mengklaim cakupan nasional.

GET /api/v1/coverage melaporkan jumlah tersimpan dan tempat terverifikasi; national_data_verified=false sengaja tidak disimpulkan dari jumlah. Sampel kurasi kota Tier-1: validasi harga/jam buka, batas administratif, izin gambar, sumber nomor darurat dan tanggal tinjauan. Impor OSM tidak otomatis membuktikan IGD, pembayaran sponsor atau kepemilikan bisnis. Tidak ada data nasional atau kontak darurat fiktif yang diaktifkan untuk memenuhi target jumlah.
