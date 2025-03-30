-- 1. Support OSM relation multipolygon candidates and fast paginated sitemap queries.
ALTER TABLE osm_import_candidates
ADD COLUMN IF NOT EXISTS way_refs bigint[];

CREATE INDEX IF NOT EXISTS places_sitemap_idx
ON places (is_active, updated_at DESC, id);

-- 2. Operator & Administrator Users Table for standard Email/Username + Password login flow.
CREATE TABLE IF NOT EXISTS users (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    email varchar(255) NOT NULL UNIQUE
        CHECK (email = lower(email) AND email ~ '^[^@\s]+@[^@\s]+\.[^@\s]+$'),
    username varchar(100) NOT NULL UNIQUE
        CHECK (username ~ '^[a-z0-9._-]{3,100}$'),
    full_name varchar(150) NOT NULL,
    password_hash text NOT NULL,
    role varchar(30) NOT NULL DEFAULT 'admin'
        CHECK (role IN ('admin', 'operator')),
    is_active boolean NOT NULL DEFAULT true,
    last_login_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS users_email_active_idx ON users(email) WHERE is_active;
CREATE INDEX IF NOT EXISTS users_username_active_idx ON users(username) WHERE is_active;
