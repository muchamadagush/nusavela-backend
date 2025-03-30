use crate::{
    error::{bad, ApiResult, AppError},
    models::{
        validate_coordinates, Category, City, DirectoryQuery, Place, SitemapPlace, SitemapQuery,
    },
    state::AppState,
};
use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

pub const CITY_FIELDS: &str =
    "id,province_id,name,short_name,slug,latitude,longitude,tier,timezone";
pub const PLACE_FIELDS: &str = "p.id,p.name,p.slug,p.address,p.latitude,p.longitude,p.phone,p.whatsapp,p.opening_hours,p.is_24_hours,p.has_emergency,p.is_verified,p.is_sponsored,p.sponsored_until,p.sponsored_tier,p.rating_avg,p.description,p.image_url,p.ticket_price_min,p.ticket_price_max,p.affiliate_hotel_url,p.affiliate_ticket_url";

pub async fn cities(State(s): State<AppState>) -> ApiResult<Vec<City>> {
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {CITY_FIELDS} FROM cities WHERE is_active ORDER BY tier,name,id"
        ))
        .fetch_all(&s.pool)
        .await?,
    ))
}

pub async fn city(State(s): State<AppState>, Path(slug): Path<String>) -> ApiResult<City> {
    Ok(Json(find_city(&s.pool, &slug).await?))
}

pub async fn find_city(pool: &sqlx::PgPool, slug: &str) -> Result<City, AppError> {
    sqlx::query_as(&format!(
        "SELECT {CITY_FIELDS} FROM cities WHERE slug=$1 AND is_active"
    ))
    .bind(slug)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NearestQuery {
    pub lat: f64,
    pub lng: f64,
}

pub async fn nearest(State(s): State<AppState>, Query(q): Query<NearestQuery>) -> ApiResult<City> {
    validate_coordinates(q.lat, q.lng)?;
    Ok(Json(
        sqlx::query_as(&format!(
            "SELECT {CITY_FIELDS} FROM cities WHERE is_active AND ST_DWithin(geom,ST_SetSRID(ST_MakePoint($1,$2),4326)::geography,500000) ORDER BY geom <-> ST_SetSRID(ST_MakePoint($1,$2),4326)::geography,id LIMIT 1"
        ))
        .bind(q.lng)
        .bind(q.lat)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::NotFound)?,
    ))
}

pub async fn categories(State(s): State<AppState>) -> ApiResult<Vec<Category>> {
    Ok(Json(
        sqlx::query_as("SELECT id,name,slug FROM categories ORDER BY sort_order,id")
            .fetch_all(&s.pool)
            .await?,
    ))
}

pub async fn directory(
    State(s): State<AppState>,
    Path((city_slug, cat_slug)): Path<(String, String)>,
    Query(q): Query<DirectoryQuery>,
) -> ApiResult<Vec<Place>> {
    q.validate()?;
    let city = find_city(&s.pool, &city_slug).await?;
    let category: i32 = sqlx::query_scalar("SELECT id FROM categories WHERE slug=$1")
        .bind(&cat_slug)
        .fetch_optional(&s.pool)
        .await?
        .ok_or(AppError::NotFound)?;
    let sql = format!(
        r#"SELECT {PLACE_FIELDS},
      CASE WHEN $3::float8 IS NOT NULL THEN ST_Distance(p.geom,ST_SetSRID(ST_MakePoint($4,$3),4326)::geography) ELSE NULL END AS distance_meters
      FROM public_places p
      WHERE p.city_id=$1 AND p.category_id=$2 AND p.is_active
        AND (NOT $5 OR p.is_24_hours) AND (NOT $6 OR (p.has_emergency AND p.is_24_hours))
        AND ($3::float8 IS NULL OR ST_DWithin(p.geom,ST_SetSRID(ST_MakePoint($4,$3),4326)::geography,$7))
      ORDER BY p.is_sponsored DESC,p.sponsored_tier DESC,
        CASE WHEN $8='rating' THEN p.rating_avg END DESC NULLS LAST,
        CASE WHEN $8='distance' THEN ST_Distance(p.geom,ST_SetSRID(ST_MakePoint($4,$3),4326)::geography) END ASC NULLS LAST,
        p.rating_avg DESC,p.id
      LIMIT 60 OFFSET $9"#
    );
    Ok(Json(
        sqlx::query_as(&sql)
            .bind(city.id)
            .bind(category)
            .bind(q.lat)
            .bind(q.lng)
            .bind(q.is_24_hours.unwrap_or(false))
            .bind(q.emergency_only.unwrap_or(false))
            .bind(q.radius_meters.unwrap_or(25000.))
            .bind(q.sort.as_deref().unwrap_or("distance"))
            .bind((q.page.unwrap_or(1) - 1) * 60)
            .fetch_all(&s.pool)
            .await?,
    ))
}

pub async fn detail(
    State(s): State<AppState>,
    Path((city, cat, place)): Path<(String, String, String)>,
) -> ApiResult<Place> {
    let sql = format!(
        "SELECT {PLACE_FIELDS},NULL::float8 AS distance_meters FROM public_places p JOIN cities c ON p.city_id=c.id JOIN categories cat ON p.category_id=cat.id WHERE c.slug=$1 AND cat.slug=$2 AND p.slug=$3 AND p.is_active AND c.is_active"
    );
    Ok(Json(
        sqlx::query_as(&sql)
            .bind(city)
            .bind(cat)
            .bind(place)
            .fetch_optional(&s.pool)
            .await?
            .ok_or(AppError::NotFound)?,
    ))
}

pub async fn sitemap_places(
    State(s): State<AppState>,
    Query(q): Query<SitemapQuery>,
) -> ApiResult<Vec<SitemapPlace>> {
    let page = q.validate_page()?;
    let limit = 5000_i64;
    let offset = (page - 1) * limit;
    let items = sqlx::query_as::<_, SitemapPlace>(
        r#"SELECT c.slug AS city_slug, cat.slug AS category_slug, p.slug AS place_slug, p.updated_at
           FROM public_places p
           JOIN cities c ON c.id = p.city_id
           JOIN categories cat ON cat.id = p.category_id
           WHERE p.is_active AND c.is_active
           ORDER BY p.is_sponsored DESC, p.updated_at DESC, p.id
           LIMIT $1 OFFSET $2"#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(items))
}

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct EventQuery {
    pub from: Option<chrono::DateTime<chrono::Utc>>,
    pub until: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn events(
    State(s): State<AppState>,
    Path(slug): Path<String>,
    Query(q): Query<EventQuery>,
) -> ApiResult<serde_json::Value> {
    let c = find_city(&s.pool, &slug).await?;
    let from = q.from.unwrap_or_else(chrono::Utc::now);
    let until = q.until.unwrap_or(from + chrono::Duration::days(31));
    if until < from || until - from > chrono::Duration::days(366) {
        return Err(bad("Invalid event date range"));
    }
    let items: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT to_jsonb(e) FROM events e WHERE city_id=$1 AND is_active AND end_datetime >= $2 AND start_datetime <= $3 ORDER BY is_sponsored DESC,start_datetime,id LIMIT 100",
    )
    .bind(c.id)
    .bind(from)
    .bind(until)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(serde_json::json!(items)))
}

pub async fn emergency(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<serde_json::Value> {
    let c = find_city(&s.pool, &slug).await?;
    let items: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT to_jsonb(e) FROM emergency_contacts e WHERE (city_id=$1 OR city_id IS NULL) AND is_active AND verified_at IS NOT NULL ORDER BY sort_order,id",
    )
    .bind(c.id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(serde_json::json!(items)))
}

pub async fn news(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<serde_json::Value> {
    let c = find_city(&s.pool, &slug).await?;
    let items: Vec<serde_json::Value> = sqlx::query_scalar(
        "SELECT to_jsonb(n) FROM news_feeds n WHERE city_id=$1 OR (city_id IS NULL AND province_id=$2) ORDER BY published_at DESC,id LIMIT 30",
    )
    .bind(c.id)
    .bind(c.province_id)
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(serde_json::json!(items)))
}

/// Counts describe stored coverage, not independent verification of national data.
pub async fn coverage(State(s): State<AppState>) -> ApiResult<serde_json::Value> {
    let counts: serde_json::Value = sqlx::query_scalar(
        "SELECT jsonb_build_object(
          'provinces', (SELECT count(*) FROM provinces),
          'active_cities', (SELECT count(*) FROM cities WHERE is_active),
          'active_places', (SELECT count(*) FROM places p JOIN cities c ON c.id=p.city_id WHERE p.is_active AND c.is_active),
          'verified_places', (SELECT count(*) FROM places p JOIN cities c ON c.id=p.city_id WHERE p.is_active AND c.is_active AND p.is_verified),
          'cities_with_places', (SELECT count(DISTINCT p.city_id) FROM places p JOIN cities c ON c.id=p.city_id WHERE p.is_active AND c.is_active),
          'verified_emergency_contacts', (SELECT count(*) FROM emergency_contacts WHERE is_active AND verified_at IS NOT NULL))",
    )
    .fetch_one(&s.pool)
    .await?;
    Ok(Json(serde_json::json!({
        "counts": counts,
        "target": {"provinces": 38, "cities": 514},
        "national_data_verified": false
    })))
}
