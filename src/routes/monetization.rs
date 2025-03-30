use crate::{
    error::{bad, ApiResult, AppError},
    models::text,
    routes::admin::authorize,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct SponsorshipPlan {
    pub code: &'static str,
    pub name_id: &'static str,
    pub name_en: &'static str,
    pub tier: i16,
    pub duration_days: i32,
    pub amount_idr: i32,
    pub features_id: &'static [&'static str],
    pub features_en: &'static [&'static str],
}

pub const PLANS: &[SponsorshipPlan] = &[
    SponsorshipPlan {
        code: "verified_monthly",
        name_id: "NusaVela Verified Partner (30 Hari)",
        name_en: "NusaVela Verified Partner (30 Days)",
        tier: 1,
        duration_days: 30,
        amount_idr: 99_000,
        features_id: &[
            "Lencana Terverifikasi & Rekomendasi Mitra",
            "Prioritas tampil di atas daftar reguler pada kategori kota",
            "Tombol kontak WhatsApp & telepon langsung",
        ],
        features_en: &[
            "Verified & Partner Recommendation Badge",
            "Priority placement above regular listings in city category",
            "Direct WhatsApp lead & phone call buttons",
        ],
    },
    SponsorshipPlan {
        code: "spotlight_monthly",
        name_id: "NusaVela City Spotlight Utama (30 Hari)",
        name_en: "NusaVela Top City Spotlight (30 Days)",
        tier: 2,
        duration_days: 30,
        amount_idr: 249_000,
        features_id: &[
            "Posisi #1 Teratas (Tier 2 Spotlight) di direktori & peta kota",
            "Pin emas khusus (#C89B3C) berukuran besar di peta interaktif",
            "Bingkai kartu Swarna Gold & lencana Rekomendasi Utama",
        ],
        features_en: &[
            "#1 Top Spotlight position in city directory & interactive map",
            "Enlarged Swarna Gold (#C89B3C) map marker",
            "Highlighted gold card frame & Top Spotlight badge",
        ],
    },
    SponsorshipPlan {
        code: "spotlight_quarterly",
        name_id: "NusaVela City Spotlight Kuartal (90 Hari · Hemat 20%)",
        name_en: "NusaVela Quarterly Spotlight (90 Days · Save 20%)",
        tier: 2,
        duration_days: 90,
        amount_idr: 599_000,
        features_id: &[
            "Seluruh keunggulan City Spotlight Tier 2 selama 90 hari penuh",
            "Prioritas indeksasi sitemap SEO & halaman beranda kota",
            "Hemat Rp148.000 dibanding perpanjangan bulanan",
        ],
        features_en: &[
            "All Tier 2 City Spotlight benefits for 90 full days",
            "Priority SEO sitemap indexing & city dashboard placement",
            "Save IDR 148,000 compared to monthly billing",
        ],
    },
];

pub fn find_plan(code: &str) -> Option<&'static SponsorshipPlan> {
    PLANS.iter().find(|p| p.code == code)
}

fn url_encode_query(input: &str) -> String {
    let mut out = String::new();
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || b"-_.".contains(&b) {
            out.push(b as char);
        } else if b == b' ' {
            out.push('+');
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

pub fn programmatic_hotel_url(
    place_name: &str,
    city_name: &str,
    lat: f64,
    lng: f64,
) -> String {
    let stay22_aid = std::env::var("STAY22_AID")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "nusavela".into());
    let query = url_encode_query(&format!("{place_name} {city_name}"));
    if let Ok(base) = std::env::var("AFFILIATE_HOTEL_TEMPLATE") {
        if base.starts_with("https://") {
            return base
                .replace("{query}", &query)
                .replace("{lat}", &format!("{lat:.6}"))
                .replace("{lng}", &format!("{lng:.6}"));
        }
    }
    format!(
        "https://www.stay22.com/allez/roam?aid={}&lat={:.6}&lng={:.6}&address={}",
        url_encode_query(&stay22_aid),
        lat,
        lng,
        query
    )
}

pub fn programmatic_ticket_url(
    place_name: &str,
    city_name: &str,
    lat: f64,
    lng: f64,
) -> String {
    let partner = std::env::var("TRAVELPAYOUTS_MARKER")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "nusavela".into());
    let query = url_encode_query(&format!("{place_name} {city_name}"));
    if let Ok(base) = std::env::var("AFFILIATE_TICKET_TEMPLATE") {
        if base.starts_with("https://") {
            return base
                .replace("{query}", &query)
                .replace("{lat}", &format!("{lat:.6}"))
                .replace("{lng}", &format!("{lng:.6}"));
        }
    }
    format!(
        "https://www.klook.com/id/search/result/?query={}&aid={}",
        query,
        url_encode_query(&partner)
    )
}

pub async fn plans() -> ApiResult<Value> {
    let stay22_configured = std::env::var("STAY22_AID")
        .ok()
        .is_some_and(|v| !v.trim().is_empty());
    let tp_configured = std::env::var("TRAVELPAYOUTS_MARKER")
        .ok()
        .is_some_and(|v| !v.trim().is_empty());
    Ok(Json(json!({
        "mode": "programmatic_effortless",
        "auto_affiliate_enabled": true,
        "stay22_configured": stay22_configured,
        "travelpayouts_configured": tp_configured,
        "plans": PLANS,
    })))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateOrderRequest {
    pub city_id: String,
    pub place_id: Uuid,
    pub business_name: String,
    pub contact_name: String,
    pub contact_phone_or_email: String,
    pub plan_code: String,
    #[serde(default)]
    pub instant_activate: bool,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SponsorshipOrderRecord {
    pub id: Uuid,
    pub city_id: String,
    pub place_id: Uuid,
    pub business_name: String,
    pub contact_name: String,
    pub contact_phone_or_email: String,
    pub plan_code: String,
    pub tier: i16,
    pub duration_days: i32,
    pub amount_idr: i32,
    pub status: String,
    pub payment_reference: String,
    pub checkout_url: String,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

pub async fn create_order(
    State(s): State<AppState>,
    Json(v): Json<CreateOrderRequest>,
) -> ApiResult<Value> {
    text(&v.city_id, 1, 10)?;
    text(&v.business_name, 2, 150)?;
    text(&v.contact_name, 2, 100)?;
    text(&v.contact_phone_or_email, 3, 100)?;
    let plan = find_plan(&v.plan_code).ok_or_else(|| bad("Invalid sponsorship plan code"))?;

    let mut tx = s.pool.begin().await?;
    // Ensure place exists, belongs to city, and is not a protected public safety category
    let cat_slug: Option<String> = sqlx::query_scalar(
        "SELECT cat.slug FROM places p JOIN categories cat ON cat.id = p.category_id WHERE p.id = $1 AND p.city_id = $2 AND p.is_active",
    )
    .bind(v.place_id)
    .bind(&v.city_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(cat_slug) = cat_slug else {
        return Err(AppError::NotFound);
    };
    if ["rumah-sakit", "layanan-publik"].contains(&cat_slug.as_str()) {
        return Err(bad("Paid sponsorship is disabled on medical and public service listings"));
    }

    let order_id = Uuid::new_v4();
    let short_ref = order_id.simple().to_string()[..10].to_uppercase();
    let payment_ref = format!("NV-INV-{short_ref}");
    let gateway_base = std::env::var("PAYMENT_GATEWAY_CHECKOUT_BASE")
        .ok()
        .filter(|u| u.starts_with("https://"))
        .unwrap_or_else(|| "https://checkout.xendit.co/web".into());
    let checkout_url = format!("{}/{payment_ref}", gateway_base.trim_end_matches('/'));

    let status = if v.instant_activate { "paid" } else { "pending" };
    let paid_at = if v.instant_activate {
        Some(Utc::now())
    } else {
        None
    };

    let order: SponsorshipOrderRecord = sqlx::query_as(
        r#"INSERT INTO sponsorship_orders(
            id, city_id, place_id, business_name, contact_name, contact_phone_or_email,
            plan_code, tier, duration_days, amount_idr, status, payment_reference, checkout_url, paid_at
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)
        RETURNING id, city_id, place_id, business_name, contact_name, contact_phone_or_email,
                  plan_code, tier, duration_days, amount_idr, status, payment_reference, checkout_url, created_at, paid_at"#,
    )
    .bind(order_id)
    .bind(&v.city_id)
    .bind(v.place_id)
    .bind(v.business_name.trim())
    .bind(v.contact_name.trim())
    .bind(v.contact_phone_or_email.trim())
    .bind(plan.code)
    .bind(plan.tier)
    .bind(plan.duration_days)
    .bind(plan.amount_idr)
    .bind(status)
    .bind(&payment_ref)
    .bind(&checkout_url)
    .bind(paid_at)
    .fetch_one(&mut *tx)
    .await?;

    if v.instant_activate {
        sqlx::query(
            r#"UPDATE places
               SET is_sponsored = true,
                   is_verified = true,
                   sponsored_tier = GREATEST(sponsored_tier, $2),
                   sponsored_until = GREATEST(COALESCE(sponsored_until, now()), now()) + make_interval(days => $3),
                   source = 'partner',
                   updated_at = now()
               WHERE id = $1"#,
        )
        .bind(v.place_id)
        .bind(plan.tier)
        .bind(plan.duration_days)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO admin_audit(actor, action, target_id, detail) VALUES('self_serve_checkout', 'auto_activate_sponsorship', $1, $2)",
        )
        .bind(v.place_id)
        .bind(json!({
            "order_id": order.id,
            "payment_reference": order.payment_reference,
            "plan_code": order.plan_code,
            "tier": order.tier,
            "duration_days": order.duration_days,
            "amount_idr": order.amount_idr
        }))
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(Json(json!({
        "order": order,
        "activated": v.instant_activate
    })))
}

pub async fn activate_order(
    State(s): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> ApiResult<Value> {
    let mut tx = s.pool.begin().await?;
    let order: Option<SponsorshipOrderRecord> = sqlx::query_as(
        r#"UPDATE sponsorship_orders
           SET status = 'paid', paid_at = COALESCE(paid_at, now())
           WHERE id = $1 AND status = 'pending'
           RETURNING id, city_id, place_id, business_name, contact_name, contact_phone_or_email,
                     plan_code, tier, duration_days, amount_idr, status, payment_reference, checkout_url, created_at, paid_at"#,
    )
    .bind(order_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(order) = order else {
        return Err(AppError::Conflict);
    };

    sqlx::query(
        r#"UPDATE places
           SET is_sponsored = true,
               is_verified = true,
               sponsored_tier = GREATEST(sponsored_tier, $2),
               sponsored_until = GREATEST(COALESCE(sponsored_until, now()), now()) + make_interval(days => $3),
               source = 'partner',
               updated_at = now()
           WHERE id = $1"#,
    )
    .bind(order.place_id)
    .bind(order.tier)
    .bind(order.duration_days)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO admin_audit(actor, action, target_id, detail) VALUES('payment_webhook', 'activate_sponsorship_order', $1, $2)",
    )
    .bind(order.place_id)
    .bind(json!({
        "order_id": order.id,
        "payment_reference": order.payment_reference,
        "plan_code": order.plan_code,
        "tier": order.tier,
        "duration_days": order.duration_days
    }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(Json(json!({
        "status": "activated",
        "order": order
    })))
}

pub async fn list_orders(
    State(s): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Vec<SponsorshipOrderRecord>> {
    authorize(&s, &headers)?;
    let rows = sqlx::query_as::<_, SponsorshipOrderRecord>(
        r#"SELECT id, city_id, place_id, business_name, contact_name, contact_phone_or_email,
                  plan_code, tier, duration_days, amount_idr, status, payment_reference, checkout_url, created_at, paid_at
           FROM sponsorship_orders
           ORDER BY created_at DESC
           LIMIT 100"#,
    )
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(rows))
}
