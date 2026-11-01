# Operasi admin dan afiliasi

Gunakan Node.js 24 untuk CLI operator; server API tetap Rust. JWT admin dibuat dengan `cargo run --locked --bin mint_admin_token -- --subject operator --seconds 900`, menggunakan ADMIN_JWT_SECRET yang sama dengan server. Isi ADMIN_API_TOKEN melalui lingkungan shell aman, bukan argumen command. CLI membaca variabel proses, bukan otomatis file .env. Jangan menyalin token ke frontend atau log bersama.

- `node scripts/admin.mjs pending`: daftar kontribusi pending (termasuk kontak privat; hanya untuk operator).
- `node scripts/admin.mjs analytics`: agregat klik 30 hari.
- `node scripts/admin.mjs review UUID /path/review.json`: payload status, notes dan patch/created_entity_id sesuai docs/API.md; tinjau data secara manual sebelum menjalankan.
- `node scripts/admin.mjs sponsor PLACE_UUID /path/sponsor.json`: tier, until dan payment_reference setelah pembayaran diverifikasi operator.
- `node scripts/admin.mjs affiliate PLACE_UUID`: gunakan AFFILIATE_HOTEL_URL dan/atau AFFILIATE_TICKET_URL berupa URL pelacakan resmi. URL yang kosong menjadi null untuk jenis tersebut; set keduanya untuk mempertahankan kedua jenis link. Konfigurasi sepenuhnya kosong ditolak agar tidak menghapus tautan tanpa sengaja.

ADMIN_API_URL default loopback, gunakan HTTPS pada staging/produksi. Token tidak ditampilkan atau disimpan CLI. Keanggotaan Traveloka/Tiket.com, parameter tracking dan verifikasi pembayaran diurus operator; contoh environment sengaja kosong. Tidak ada aktivasi komisi atau janji pendapatan otomatis.
