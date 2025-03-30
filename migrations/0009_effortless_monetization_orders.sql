-- Migration 0009: Self-Serve UMKM Sponsorship Orders & Automated Monetization Support

CREATE TABLE IF NOT EXISTS sponsorship_orders (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    city_id varchar(10) NOT NULL REFERENCES cities(id),
    place_id uuid NOT NULL REFERENCES places(id),
    business_name varchar(150) NOT NULL,
    contact_name varchar(100) NOT NULL,
    contact_phone_or_email varchar(100) NOT NULL,
    plan_code varchar(30) NOT NULL
        CHECK (plan_code IN ('verified_monthly', 'spotlight_monthly', 'spotlight_quarterly')),
    tier smallint NOT NULL CHECK (tier IN (1, 2)),
    duration_days int NOT NULL CHECK (duration_days BETWEEN 7 AND 365),
    amount_idr int NOT NULL CHECK (amount_idr > 0),
    status varchar(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'paid', 'cancelled')),
    payment_reference varchar(120) NOT NULL UNIQUE,
    checkout_url text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    paid_at timestamptz
);

CREATE INDEX IF NOT EXISTS sponsorship_orders_place_idx
    ON sponsorship_orders(place_id, status, created_at DESC);
