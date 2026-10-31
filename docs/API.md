# API v1

Semua respons JSON. Error: `{"error":{"code":"...","message":"..."}}`. Koordinat berupa angka JSON, bukan string desimal. Rute tanpa prefix: `GET /healthz` (proses hidup) dan `GET /readyz` (query database berhasil). Cache API `no-store`; caching SSR kota ditangani frontend/CDN dengan query key yang lengkap.

## Publik

| Metode | Path setelah `/api/v1` | Respons |
| --- | --- | --- |
| GET | `/cities` | Array kota aktif, urut tier/nama/ID. |
| GET | `/cities/nearest?lat=&lng=` | Kota aktif terdekat dalam 500 km; 404 jika tidak ada. |
| GET | `/cities/{slug}` | Objek kota aktif, atau 404. |
| GET | `/categories` | Array kategori. |
| GET | `/kota/{city}/{category}` | Array tempat, maksimal 60 item per halaman. |
| GET | `/kota/{city}/{category}/{place}` | Objek tempat yang harus cocok kota dan kategorinya. |
| GET | `/cities/{slug}/emergency` | Kontak nasional/kota aktif dengan sumber dan waktu verifikasi. |
| GET | `/cities/{slug}/events?from=&until=` | Event bertumpang tindih rentang UTC ISO8601; default sekarang sampai 31 hari, maksimal 366 hari. |
| GET | `/cities/{slug}/news` | Maksimal 30 headline kota/provinsi dengan sumber dan ringkasan maksimal 160 karakter. |
| GET | `/cities/{slug}/weather` | Cuaca ter-cache 30 menit, timestamp sumber dan atribusi OpenWeather. 503 jika key kosong/upstream gagal. |
| POST | `/submissions` | Buat kontribusi pending; respons `{id}`. |
| POST | `/clicks` | Catat klik tujuan yang ditentukan server; respons `{ok,target_url}`. |

Filter direktori: `lat`, `lng` berpasangan; `radius_meters` 100–100000 (default 25000); `is_24_hours` boolean literal; `emergency_only` boolean; `sort=distance|rating`; `page=1..10000`. Sponsor **aktif** selalu lebih dahulu, lalu tier, sort terpilih, rating dan ID stabil. Default sorting adalah distance. Tanpa koordinat, jarak null dan rating menjadi pemecah urutan. Filter `emergency_only=true` memerlukan has_emergency sekaligus buka 24 jam.

Objek kota: `id,province_id,name,short_name,slug,latitude,longitude,tier,timezone`.

Objek tempat: `id,name,slug,address,latitude,longitude,phone,whatsapp,opening_hours,is_24_hours,has_emergency,is_verified,is_sponsored,sponsored_until,sponsored_tier,distance_meters,rating_avg,description,image_url,ticket_price_min,ticket_price_max,affiliate_hotel_url,affiliate_ticket_url`.

Kontribusi: `{city_id,place_id?,submission_type,submitter_name,submitter_contact,payload:{message}}`. Tipe: `correction`, `business_claim`, `new_place`, `new_event`. Koreksi/klaim wajib place_id dari kota yang sama. Message 10–3000 karakter, nama maksimal 100, kontak 3–100. Payload lain dan field tak dikenal ditolak; kontak tidak pernah ditampilkan di API publik.

Klik: `{city_id,place_id,click_type}`. Tipe: `affiliate_hotel`, `affiliate_ticket`, `whatsapp_lead`, `phone_call`. Klien tidak boleh mengirim target_url; server memuatnya dari tempat aktif yang cocok dengan kota. Link affiliate wajib HTTPS tanpa userinfo. Tracking affiliate ditolak pada kategori rumah-sakit/layanan-publik. WhatsApp harus dalam format nomor internasional yang telah dikurasi operator.

## Admin

Wajib `Authorization: Bearer <JWT HS256>` dengan role admin, issuer dan audience yang dikonfigurasi.

| Metode | Path setelah `/api/v1/admin` | Body / perilaku |
| --- | --- | --- |
| GET | `/submissions` | Maksimal 100 pending, termasuk kontak untuk operator berwenang. |
| POST | `/submissions/{id}/review` | `{status:'approved'|'rejected',notes,patch?,created_entity_id?}`. Transaksi + row lock + audit. |
| POST | `/places` | Buat tempat terkurasi: `{city_id,category_slug,name,slug,latitude,longitude,address?,description?,opening_hours?,is_24_hours?,has_emergency?,is_verified?,ticket_price_min?,ticket_price_max?}`. |
| POST | `/events` | `{city_id,title,slug,venue_name?,address?,start_datetime,end_datetime,ticket_price?,description?}`. |
| POST | `/emergency` | `{city_id?,service_name,category,phone_number,source_url,is_toll_free?,is_24_hours?}`. Admin menyatakan sumber telah diverifikasi; sistem menyimpan waktu verifikasi. |
| PATCH | `/places/{id}/sponsorship` | `{tier:0|1|2,until?,payment_reference}`; tier 0 mematikan sponsor, tier positif harus expiry masa depan maksimal 366 hari. |
| PATCH | `/places/{id}/affiliate` | `{hotel_url:string|null,ticket_url:string|null}`; null menghapus link. |
| GET | `/analytics` | Agregat klik 30 hari per kota dan tipe. |

Patch review hanya menerima `description,opening_hours,phone,whatsapp,is_24_hours,has_emergency,is_verified`. Field absent mempertahankan nilai lama; string kosong dapat membersihkan teks. Tidak ada pengubahan sponsor atau SQL dari payload publik. Review new_place/new_event memerlukan entitas yang telah dibuat operator dan cocok kota; pesan publik tidak dieksekusi sebagai data administratif otomatis.

Status penting: 400 validasi, 401 admin tidak sah, 404 sumber tidak ada, 409 duplikat/sudah direview, 413 body terlalu besar, 422 JSON tidak sesuai skema, 429 batas request, 503 database/upstream tidak tersedia. Body maksimal 16 KiB. Default timeout request 20 detik, statement SQL 10 detik untuk koneksi server utama. Semua write dibatasi 30 per menit per socket peer melalui PostgreSQL. Proxy perlu pembatasan per pengguna/IP asli dan batas GET sendiri; forwarded header tidak dipercaya otomatis.
