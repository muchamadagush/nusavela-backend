-- Activate verified national emergency contacts with official Indonesian government sources.
UPDATE emergency_contacts
SET is_active = true,
    is_toll_free = true,
    is_24_hours = true,
    verified_at = now(),
    source_url = CASE phone_number
        WHEN '112' THEN 'https://layanan112.komdigi.go.id/'
        WHEN '119' THEN 'https://kemkes.go.id/'
        WHEN '110' THEN 'https://polri.go.id/'
        WHEN '113' THEN 'https://pemadam.jakarta.go.id/'
        WHEN '115' THEN 'https://basarnas.go.id/'
        ELSE 'https://indonesia.go.id/'
    END,
    service_name = CASE phone_number
        WHEN '112' THEN 'Panggilan Darurat Terpadu Nasional (112)'
        WHEN '119' THEN 'Ambulans & Gawat Darurat Medis Kemenkes (119)'
        WHEN '110' THEN 'Layanan Darurat Kepolisian RI (110)'
        WHEN '113' THEN 'Pemadam Kebakaran & Penyelamatan (113)'
        WHEN '115' THEN 'Pencarian & Pertolongan Basarnas (115)'
        ELSE service_name
    END
WHERE city_id IS NULL AND phone_number IN ('112', '119', '110', '113', '115');
