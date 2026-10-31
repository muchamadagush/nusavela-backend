CREATE EXTENSION IF NOT EXISTS postgis;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE provinces (
    id varchar(10) PRIMARY KEY,
    name varchar(100) NOT NULL,
    slug varchar(150) NOT NULL UNIQUE,
    rss_news_urls text[] NOT NULL DEFAULT '{}',
    created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE cities (
    id varchar(10) PRIMARY KEY,
    province_id varchar(10) NOT NULL REFERENCES provinces(id),
    name varchar(150) NOT NULL,
    short_name varchar(100) NOT NULL,
    slug varchar(150) UNIQUE NOT NULL,
    tier smallint NOT NULL DEFAULT 2 CHECK (tier IN (1,2)),
    latitude double precision NOT NULL CHECK (latitude BETWEEN -90 AND 90),
    longitude double precision NOT NULL CHECK (longitude BETWEEN -180 AND 180),
    geom geography(Point,4326) GENERATED ALWAYS AS
        (ST_SetSRID(ST_MakePoint(longitude,latitude),4326)::geography) STORED,
    boundary geometry(MultiPolygon,4326),
    population bigint CHECK (population >= 0),
    area_km2 numeric(12,2) CHECK (area_km2 >= 0),
    hero_image_url text,
    meta_description text,
    timezone text NOT NULL DEFAULT 'Asia/Jakarta' CHECK (timezone IN ('Asia/Jakarta','Asia/Makassar','Asia/Jayapura')),
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX cities_geom_idx ON cities USING gist(geom);
CREATE INDEX cities_boundary_idx ON cities USING gist(boundary);
CREATE INDEX cities_province_idx ON cities(province_id);
CREATE TABLE categories (
    id serial PRIMARY KEY,
    name varchar(100) NOT NULL,
    slug varchar(100) NOT NULL UNIQUE,
    icon_name varchar(50),
    osm_tags jsonb NOT NULL DEFAULT '{}',
    sort_order smallint NOT NULL DEFAULT 0
);
INSERT INTO categories(name,slug,icon_name,sort_order) VALUES
('Wisata','wisata','compass',1),('Rumah Sakit & Faskes','rumah-sakit','hospital',2),
('Kuliner & UMKM','kuliner','utensils',3),('Transportasi','transportasi','train',4),
('Layanan Publik','layanan-publik','landmark',5);
CREATE TABLE places (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    osm_type varchar(10) CHECK (osm_type IN ('node','way','relation')),
    osm_id bigint,
    city_id varchar(10) NOT NULL REFERENCES cities(id),
    category_id int NOT NULL REFERENCES categories(id),
    name varchar(255) NOT NULL,
    slug varchar(300) NOT NULL,
    address text,
    latitude double precision NOT NULL CHECK (latitude BETWEEN -90 AND 90),
    longitude double precision NOT NULL CHECK (longitude BETWEEN -180 AND 180),
    geom geography(Point,4326) GENERATED ALWAYS AS
        (ST_SetSRID(ST_MakePoint(longitude,latitude),4326)::geography) STORED,
    phone varchar(50), whatsapp varchar(50), website_url text,
    opening_hours varchar(255), is_24_hours boolean NOT NULL DEFAULT false,
    has_emergency boolean NOT NULL DEFAULT false,
    ticket_price_min int CHECK (ticket_price_min >= 0),
    ticket_price_max int CHECK (ticket_price_max >= 0 AND (ticket_price_min IS NULL OR ticket_price_max >= ticket_price_min)),
    rating_avg numeric(3,2) NOT NULL DEFAULT 0 CHECK (rating_avg BETWEEN 0 AND 5),
    review_count int NOT NULL DEFAULT 0 CHECK (review_count >= 0),
    description text, image_url text,
    is_verified boolean NOT NULL DEFAULT false,
    is_sponsored boolean NOT NULL DEFAULT false,
    sponsored_until timestamptz,
    sponsored_tier smallint NOT NULL DEFAULT 0 CHECK (sponsored_tier BETWEEN 0 AND 2),
    affiliate_hotel_url text CHECK (affiliate_hotel_url IS NULL OR affiliate_hotel_url ~ '^https://'),
    affiliate_ticket_url text CHECK (affiliate_ticket_url IS NULL OR affiliate_ticket_url ~ '^https://'),
    source varchar(20) NOT NULL DEFAULT 'osm' CHECK (source IN ('osm','admin','community','partner')),
    osm_snapshot jsonb,
    city_assignment varchar(30) NOT NULL DEFAULT 'verified' CHECK (city_assignment IN ('boundary','nearest_centroid','verified')),
    is_active boolean NOT NULL DEFAULT true,
    last_verified_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(), updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(city_id,slug), UNIQUE(osm_type,osm_id),
    CHECK ((osm_id IS NULL) = (osm_type IS NULL)),
    CHECK (NOT is_sponsored OR (sponsored_until IS NOT NULL AND sponsored_tier > 0))
);
CREATE INDEX places_geom_idx ON places USING gist(geom);
CREATE INDEX places_directory_idx ON places(city_id,category_id,is_active);
CREATE INDEX places_sponsors_idx ON places(city_id,is_sponsored,sponsored_tier);
CREATE VIEW public_places AS SELECT p.id,p.city_id,p.category_id,p.name,p.slug,p.address,
 p.latitude,p.longitude,p.geom,p.phone,p.whatsapp,p.website_url,p.opening_hours,
 p.is_24_hours,p.has_emergency,p.ticket_price_min,p.ticket_price_max,p.rating_avg::float8 AS rating_avg,
 p.review_count,p.description,p.image_url,p.is_verified,p.sponsored_until,
 (p.is_sponsored AND p.sponsored_until > now()) AS is_sponsored,
 CASE WHEN p.is_sponsored AND p.sponsored_until > now() THEN p.sponsored_tier ELSE 0 END::smallint AS sponsored_tier,
 p.affiliate_hotel_url,p.affiliate_ticket_url,p.source,p.is_active,p.updated_at
 FROM places p;
CREATE TABLE emergency_contacts (
    id serial PRIMARY KEY, city_id varchar(10) REFERENCES cities(id),
    service_name varchar(150) NOT NULL, category varchar(50) NOT NULL,
    phone_number varchar(50) NOT NULL, is_toll_free boolean NOT NULL DEFAULT false,
    is_24_hours boolean NOT NULL DEFAULT true, sort_order smallint NOT NULL DEFAULT 0,
    is_active boolean NOT NULL DEFAULT false,
    source_url text, verified_at timestamptz,
    CHECK (NOT is_active OR (source_url IS NOT NULL AND verified_at IS NOT NULL))
);
CREATE INDEX emergency_city_idx ON emergency_contacts(city_id);
-- National emergency numbers are deliberately not activated without a verified source.
CREATE TABLE events (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(), city_id varchar(10) NOT NULL REFERENCES cities(id),
    title varchar(255) NOT NULL, slug varchar(300) NOT NULL, venue_name varchar(255), address text,
    latitude double precision CHECK (latitude BETWEEN -90 AND 90),
    longitude double precision CHECK (longitude BETWEEN -180 AND 180),
    start_datetime timestamptz NOT NULL, end_datetime timestamptz NOT NULL,
    ticket_price int NOT NULL DEFAULT 0 CHECK (ticket_price >= 0), ticket_affiliate_url text,
    organizer_name varchar(150), poster_url text, description text,
    is_sponsored boolean NOT NULL DEFAULT false, is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(), UNIQUE(city_id,slug),
    CHECK(end_datetime >= start_datetime), CHECK((latitude IS NULL) = (longitude IS NULL))
);
CREATE INDEX events_city_date_idx ON events(city_id,start_datetime);
CREATE TABLE news_feeds (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(), province_id varchar(10) REFERENCES provinces(id),
    city_id varchar(10) REFERENCES cities(id), title varchar(300) NOT NULL,
    summary varchar(160), source_name varchar(100) NOT NULL, source_url text NOT NULL UNIQUE,
    thumbnail_url text, published_at timestamptz NOT NULL, created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (source_url ~ '^https://')
);
CREATE INDEX news_city_idx ON news_feeds(city_id,published_at DESC);
CREATE INDEX news_province_idx ON news_feeds(province_id,published_at DESC);
CREATE TABLE community_submissions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(), city_id varchar(10) NOT NULL REFERENCES cities(id),
    place_id uuid REFERENCES places(id), submission_type varchar(50) NOT NULL
      CHECK (submission_type IN ('correction','business_claim','new_place','new_event')),
    submitter_name varchar(100) NOT NULL, submitter_contact varchar(100) NOT NULL,
    payload jsonb NOT NULL CHECK (jsonb_typeof(payload) = 'object'),
    status varchar(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','rejected')),
    admin_notes text, created_at timestamptz NOT NULL DEFAULT now(), reviewed_at timestamptz, reviewed_by text
);
CREATE INDEX submissions_pending_idx ON community_submissions(status,created_at);
CREATE TABLE monetization_clicks (
    id bigserial PRIMARY KEY, city_id varchar(10) REFERENCES cities(id) ON DELETE SET NULL,
    place_id uuid REFERENCES places(id) ON DELETE SET NULL,
    click_type varchar(50) NOT NULL CHECK (click_type IN ('affiliate_hotel','affiliate_ticket','whatsapp_lead','phone_call')),
    target_url text NOT NULL, clicked_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX clicks_place_date_idx ON monetization_clicks(place_id,clicked_at);
CREATE TABLE admin_audit (
    id bigserial PRIMARY KEY, actor text NOT NULL, action text NOT NULL,
    target_id uuid NOT NULL, detail jsonb NOT NULL, created_at timestamptz NOT NULL DEFAULT now()
);
CREATE TABLE request_limits (
    bucket text PRIMARY KEY, window_start timestamptz NOT NULL DEFAULT now(), count int NOT NULL DEFAULT 1
);
