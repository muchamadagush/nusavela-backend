-- Migration 0008: Wonderful Nusantara Curated Showcase Places & Cultural Events Across All 6 Island Regions

INSERT INTO places (
    city_id, category_id, name, slug, address, latitude, longitude,
    phone, whatsapp, opening_hours, is_24_hours, has_emergency,
    ticket_price_min, ticket_price_max, rating_avg, review_count,
    description, image_url, is_verified, source, city_assignment, is_active, last_verified_at
)
SELECT
    v.city_id, cat.id, v.name, v.slug, v.address, v.latitude, v.longitude,
    v.phone, v.whatsapp, v.opening_hours, v.is_24_hours, v.has_emergency,
    v.ticket_price_min, v.ticket_price_max, v.rating_avg, v.review_count,
    v.description, v.image_url, true, 'admin', 'verified', true, now()
FROM (
    VALUES
        -- BALI & NUSA TENGGARA: Labuan Bajo / Manggarai Barat (5315), Badung (5103), Gianyar (5104), Mataram (5271)
        ('5315', 'wisata', 'Taman Nasional Komodo & Pulau Padar', 'taman-nasional-komodo-pulau-padar', 'Kawasan Taman Nasional Komodo, Labuan Bajo, Kab. Manggarai Barat', -8.659400, 119.584400, '+6238541005', NULL, 'Setiap Hari 06:00-18:00 WITA', false, false, 150000, 450000, 4.9, 5200, 'Situs Warisan Dunia UNESCO rumah bagi satwa purba Komodo, puncak panorama tiga teluk Pulau Padar, dan Pink Beach.', 'https://images.unsplash.com/photo-1518548419970-58e3b4079ab2?auto=format&fit=crop&w=1000&q=80'),
        ('5315', 'rumah-sakit', 'RSUD Komodo Labuan Bajo', 'rsud-komodo-labuan-bajo', 'Jl. Trans Flores, Desa Golo Bilas, Kec. Komodo, Kab. Manggarai Barat', -8.523600, 119.898100, '+623852443355', NULL, 'IGD & Trauma Center 24 Jam', true, true, NULL, NULL, 4.6, 640, 'Rumah sakit rujukan utama kawasan pariwisata super prioritas Labuan Bajo dengan fasilitas IGD dan hiperbarik 24 jam.', NULL),
        ('5103', 'wisata', 'Pura Luhur Uluwatu & Tebing Selatan', 'pura-luhur-uluwatu-badung', 'Pecatu, Kec. Kuta Selatan, Kabupaten Badung, Bali', -8.829140, 115.084900, NULL, NULL, 'Setiap Hari 07:00-19:00 WITA', false, false, 50000, 150000, 4.8, 6100, 'Pura laut megah di ujung tebing karang setinggi 70 meter dengan panorama matahari terbenam Samudra Hindia dan Tari Kecak Api.', 'https://images.unsplash.com/photo-1537996194471-e657df975ab4?auto=format&fit=crop&w=1000&q=80'),
        ('5104', 'wisata', 'Terasering Sawah Tegallalang & Ubud', 'terasering-sawah-tegallalang-gianyar', 'Jl. Raya Tegallalang, Kec. Tegallalang, Kabupaten Gianyar, Bali', -8.431200, 115.279200, NULL, NULL, 'Setiap Hari 08:00-18:00 WITA', false, false, 25000, 50000, 4.7, 4300, 'Lanskap persawahan berundak sistem pengairan tradisional Subak khas Bali yang diakui sebagai warisan budaya dunia.', 'https://images.unsplash.com/photo-1555400038-63f5ba517a47?auto=format&fit=crop&w=1000&q=80'),

        -- MALUKU & PAPUA: Raja Ampat (9605), Kota Ambon (8171)
        ('9603', 'wisata', 'Kepulauan Karst Piaynemo & Telaga Bintang', 'piaynemo-telaga-bintang-raja-ampat', 'Distrik Waigeo Barat Kepulauan, Kabupaten Raja Ampat, Papua Barat Daya', -0.564200, 130.270800, NULL, NULL, 'Setiap Hari 06:00-18:00 WIT', false, false, 300000, 1000000, 4.9, 3890, 'Gugusan pulau karst zambrud di atas laguna pirus jernih yang menjadi pusat keanekaragaman hayati terumbu karang tertinggi di dunia.', 'https://images.unsplash.com/photo-1516690561799-46d8f74f9abf?auto=format&fit=crop&w=1000&q=80'),
        ('8180', 'wisata', 'Pantai Pintu Kota & Teluk Ambon', 'pantai-pintu-kota-ambon', 'Dusun Airlouw, Nusaniwe, Kota Ambon, Maluku', -3.770500, 128.146200, NULL, NULL, 'Setiap Hari 07:00-18:00 WIT', false, false, 10000, 20000, 4.7, 820, 'Tebing karang alami berlubang yang menghadap langsung ke Laut Banda dengan deburan ombak dan panorama bahari khas Ambon Manise.', NULL),

        -- SULAWESI: Kota Manado / Bunaken (7171), Kota Makassar (7371)
        ('7182', 'wisata', 'Taman Nasional Bunaken', 'taman-nasional-bunaken-manado', 'Kepulauan Bunaken, Kec. Bunaken Kepulauan, Kota Manado, Sulawesi Utara', 1.623100, 124.762100, '+62431825365', NULL, 'Setiap Hari 24 Jam', true, false, 5000, 150000, 4.9, 2940, 'Taman laut tropis legendaris Sulawesi Utara dengan dinding karang vertikal raksasa, penyu hijau, dan ratusan spesies ikan karang.', 'https://images.unsplash.com/photo-1544551763-46a013bb70d5?auto=format&fit=crop&w=1000&q=80'),
        ('7392', 'wisata', 'Benteng Fort Rotterdam & Pantai Losari', 'benteng-fort-rotterdam-makassar', 'Jl. Ujung Pandang, Bulo Gading, Kec. Ujung Pandang, Kota Makassar', -5.133800, 119.405300, NULL, NULL, 'Setiap Hari 08:00-18:00 WITA', false, false, 10000, 20000, 4.7, 2610, 'Benteng peninggalan Kerajaan Gowa-Tallo abad ke-16 berbentuk penyu yang menyimpan Museum La Galigo di tepi pesisir Makassar.', NULL),

        -- SUMATERA: Samosir / Danau Toba (1217), Kota Medan (1271), Kota Bukittinggi (1375)
        ('1217', 'wisata', 'Kaldera Danau Toba & Desa Huta Siallagan', 'kaldera-danau-toba-samosir', 'Ambarita, Kec. Simanindo, Kabupaten Samosir, Sumatera Utara', 2.678900, 98.831200, NULL, NULL, 'Setiap Hari 08:00-18:00 WIB', false, false, 10000, 25000, 4.8, 3150, 'UNESCO Global Geopark dan danau vulkanik terbesar di dunia dengan perkampungan rumah adat Batak Bolon di Pulau Samosir.', 'https://images.unsplash.com/photo-1588668214407-6ea9a6d8c272?auto=format&fit=crop&w=1000&q=80'),
        ('1296', 'wisata', 'Istana Maimun Kesultanan Deli', 'istana-maimun-medan', 'Jl. Brigjen Katamso No.66, Aur, Kec. Medan Maimun, Kota Medan', 3.575200, 98.683800, '+62614536883', NULL, 'Setiap Hari 08:00-17:00 WIB', false, false, 10000, 15000, 4.6, 2890, 'Istana kebesaran Kesultanan Deli tahun 1888 dengan perpaduan arsitektur Melayu, Islam, Timur Tengah, dan Eropa.', NULL),
        ('1387', 'wisata', 'Jam Gadang & Taman Panorama Ngarai Sianok', 'jam-gadang-ngarai-sianok-bukittinggi', 'Benteng Pasar Atas, Kec. Guguk Panjang, Kota Bukittinggi, Sumatera Barat', -0.305100, 100.369200, NULL, NULL, 'Setiap Hari 24 Jam', true, false, 0, 15000, 4.8, 3720, 'Menara jam bersejarah ikon dataran tinggi Minangkabau yang berdekatan dengan lembah curam Ngarai Sianok dan Lubang Jepang.', NULL),

        -- KALIMANTAN: Kota Singkawang (6184), Kota Balikpapan (6471)
        ('6184', 'wisata', 'Kawasan Pusaka Vihara Tri Dharma Bumi Raya', 'vihara-tri-dharma-bumi-raya-singkawang', 'Jl. Sejahtera, Melayu, Kec. Singkawang Barat, Kota Singkawang', 0.906800, 108.987200, NULL, NULL, 'Setiap Hari 06:00-21:00 WIB', false, false, 0, 0, 4.8, 1480, 'Jantung budaya Kota Seribu Klenteng Singkawang yang menjadi pusat perayaan Festival Cap Go Meh dan harmoni akulturasi Nusantara.', NULL),

        -- JAWA: Kota Malang (3573), Kota Surakarta (3372), Kota Semarang (3374)
        ('35102', 'wisata', 'Kampung Warna-Warni Jodipan & Gerbang Bromo', 'kampung-warna-warni-jodipan-malang', 'Kel. Jodipan, Kec. Blimbing, Kota Malang, Jawa Timur', -7.983100, 112.637800, NULL, NULL, 'Setiap Hari 07:00-18:00 WIB', false, false, 5000, 10000, 4.6, 2190, 'Destinasi kreatif tepi Sungai Brantas dengan jembatan kaca dan titik keberangkatan favorit menuju Taman Nasional Bromo Tengger Semeru.', 'https://images.unsplash.com/photo-1588668214407-6ea9a6d8c272?auto=format&fit=crop&w=1000&q=80'),
        ('33101', 'wisata', 'Pura Mangkunegaran & Pasar Triwindu', 'pura-mangkunegaran-surakarta', 'Jl. Ronggowarsito, Keprabon, Kec. Banjarsari, Kota Surakarta', -7.567200, 110.823100, '+62271642016', NULL, 'Senin-Minggu 08:30-15:00 WIB', false, false, 30000, 50000, 4.8, 2540, 'Istana kadipaten Jawa tahun 1757 dengan Pendopo Ageng megah, koleksi gamelan pusaka, dan taman arsitektur klasik di jantung Kota Solo.', NULL),
        ('33103', 'wisata', 'Kawasan Kota Lama Semarang & Gereja Blenduk', 'kawasan-kota-lama-semarang', 'Jl. Letjen Suprapto, Tanjung Mas, Kec. Semarang Utara, Kota Semarang', -6.968200, 110.427500, NULL, NULL, 'Setiap Hari 24 Jam', true, false, 0, 10000, 4.8, 4120, 'Kawasan cagar budaya berjuluk Little Netherland dengan gedung kolonial terestorasi, galeri seni kontemporer, dan ruang pejalan kaki ramah wisata.', NULL)
) AS v(
    city_id, cat_slug, name, slug, address, latitude, longitude,
    phone, whatsapp, opening_hours, is_24_hours, has_emergency,
    ticket_price_min, ticket_price_max, rating_avg, review_count, description, image_url
)
JOIN categories cat ON cat.slug = v.cat_slug
ON CONFLICT (city_id, slug) DO UPDATE SET
    image_url = COALESCE(EXCLUDED.image_url, places.image_url),
    description = EXCLUDED.description,
    updated_at = now();

-- Enrich image_url for existing Bandung, Jakarta, Yogyakarta, Surabaya, and Denpasar landmarks
UPDATE places SET image_url = 'https://images.unsplash.com/photo-1555899434-94d1368aa7af?auto=format&fit=crop&w=1000&q=80' WHERE slug = 'gedung-sate-bandung' AND image_url IS NULL;
UPDATE places SET image_url = 'https://images.unsplash.com/photo-1518548419970-58e3b4079ab2?auto=format&fit=crop&w=1000&q=80' WHERE slug = 'kawah-putih-ciwidey' AND image_url IS NULL;
UPDATE places SET image_url = 'https://images.unsplash.com/photo-1584810359583-96fc3448beaa?auto=format&fit=crop&w=1000&q=80' WHERE slug = 'keraton-yogyakarta' AND image_url IS NULL;
UPDATE places SET image_url = 'https://images.unsplash.com/photo-1537996194471-e657df975ab4?auto=format&fit=crop&w=1000&q=80' WHERE slug = 'monumen-bajra-sandhi-denpasar' AND image_url IS NULL;

-- Seed upcoming regional festivals & events (relative to now() so they always appear in the 31-day window)
INSERT INTO events (
    city_id, title, slug, venue_name, address, latitude, longitude,
    start_datetime, end_datetime, ticket_price, organizer_name, description, is_sponsored, is_active
)
VALUES
    ('3291', 'Asia Africa & Braga Heritage Festival', 'asia-africa-braga-heritage-festival', 'Koridor Jalan Braga & Asia Afrika, Kota Bandung', 'Jl. Asia Afrika, Kec. Sumur Bandung, Kota Bandung', -6.921100, 107.607500, now() + interval '5 days', now() + interval '7 days', 0, 'Dinas Kebudayaan dan Pariwisata Kota Bandung', 'Parade budaya lintas bangsa, pameran kopi khas Priangan, instalasi seni Art Deco, dan pertunjukan musik jalanan.', false, true),
    ('3475', 'Sekaten & Grebeg Budaya Ngayogyakarta', 'sekaten-grebeg-budaya-yogyakarta', 'Alun-Alun Utara Keraton Yogyakarta', 'Panembahan, Kec. Kraton, Kota Yogyakarta', -7.803900, 110.364400, now() + interval '8 days', now() + interval '12 days', 0, 'Keraton Ngayogyakarta & Dinas Pariwisata DIY', 'Pekan budaya tradisi gamelan pusaka, pasar rakyat Sekaten, dan pertunjukan seni klasik Yogyakarta.', false, true),
    ('5171', 'Pesta Kesenian & Kuliner Nusantara Bali', 'pesta-kesenian-kuliner-bali-denpasar', 'Taman Budaya Art Centre Werdi Budaya, Denpasar', 'Jl. Nusa Indah No.1, Panjer, Kota Denpasar', -8.655200, 115.234100, now() + interval '4 days', now() + interval '10 days', 0, 'Dinas Kebudayaan Provinsi Bali', 'Pagelaran gong kebyar, pameran kriya tenun endek, dan festival gastronomi tradisional Pulau Dewata.', false, true),
    ('35107', 'Surabaya Cross Culture & Jazz Pesisir', 'surabaya-cross-culture-jazz-pesisir', 'Balai Pemuda & Alun-Alun Surabaya', 'Jl. Gubernur Suryo No.15, Embong Kaliasin, Kota Surabaya', -7.263800, 112.745300, now() + interval '6 days', now() + interval '8 days', 0, 'Disbudporapar Kota Surabaya', 'Festival seni lintas budaya nusantara, panggung musik jazz malam hari, dan bazar UMKM kuliner Jawa Timur.', false, true),
    ('3172', 'Festival Kota Tua & Nusantara Gastronomy', 'festival-kota-tua-nusantara-gastronomy', 'Lapangan Banteng & Kawasan Gambir, Jakarta Pusat', 'Gambir, Kota Administrasi Jakarta Pusat', -6.170200, 106.835100, now() + interval '3 days', now() + interval '6 days', 0, 'Dinas Pariwisata dan Ekonomi Kreatif DKI Jakarta', 'Pameran kuliner warisan rempah Nusantara, tur sejarah arsitektur, dan pertunjukan orkestra senja.', false, true)
ON CONFLICT (city_id, slug) DO UPDATE SET
    start_datetime = EXCLUDED.start_datetime,
    end_datetime = EXCLUDED.end_datetime,
    is_active = true;

