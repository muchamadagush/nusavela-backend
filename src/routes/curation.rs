use crate::{
    error::{bad, ApiResult, AppError},
    models::{https_url, sanitize_phone_digits, text, validate_coordinates, validate_slug},
    routes::admin::authorize,
    state::AppState,
};
use axum::{extract::State, http::HeaderMap, Json};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewPlace {
    pub city_id: String,
    pub category_slug: String,
    pub name: String,
    pub slug: String,
    pub latitude: f64,
    pub longitude: f64,
    pub address: Option<String>,
    pub description: Option<String>,
    pub opening_hours: Option<String>,
    #[serde(default)]
    pub is_24_hours: bool,
    #[serde(default)]
    pub has_emergency: bool,
    #[serde(default)]
    pub is_verified: bool,
    pub ticket_price_min: Option<i32>,
    pub ticket_price_max: Option<i32>,
}

pub async fn create_place(
    State(s): State<AppState>,
    h: HeaderMap,
    Json(v): Json<NewPlace>,
) -> ApiResult<Value> {
    let admin = authorize(&s, &h)?;
    text(&v.name, 1, 255)?;
    validate_slug(&v.slug)?;
    validate_coordinates(v.latitude, v.longitude)?;
    for (t, max) in [
        (&v.address, 2000),
        (&v.description, 10000),
        (&v.opening_hours, 255),
    ] {
        if let Some(t) = t {
            text(t, 0, max)?;
        }
    }
    if v.ticket_price_min.is_some_and(|p| p < 0)
        || v.ticket_price_max.is_some_and(|p| p < 0)
        || matches!((v.ticket_price_min, v.ticket_price_max), (Some(a), Some(b)) if a > b)
    {
        return Err(bad("Invalid ticket price range"));
    }
    let mut tx = s.pool.begin().await?;
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO places(city_id,category_id,name,slug,latitude,longitude,address,description,opening_hours,is_24_hours,has_emergency,is_verified,ticket_price_min,ticket_price_max,source,last_verified_at) SELECT c.id,cat.id,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,'admin',now() FROM cities c CROSS JOIN categories cat WHERE c.id=$1 AND c.is_active AND cat.slug=$2 RETURNING places.id",
    )
    .bind(v.city_id)
    .bind(v.category_slug)
    .bind(v.name)
    .bind(v.slug)
    .bind(v.latitude)
    .bind(v.longitude)
    .bind(v.address)
    .bind(v.description)
    .bind(v.opening_hours)
    .bind(v.is_24_hours)
    .bind(v.has_emergency)
    .bind(v.is_verified)
    .bind(v.ticket_price_min)
    .bind(v.ticket_price_max)
    .fetch_optional(&mut *tx)
    .await?;
    let id = id.ok_or(AppError::NotFound)?;
    audit(&mut tx, &admin.sub, "create_place", id).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewEvent {
    pub city_id: String,
    pub title: String,
    pub slug: String,
    pub venue_name: Option<String>,
    pub address: Option<String>,
    pub start_datetime: DateTime<Utc>,
    pub end_datetime: DateTime<Utc>,
    #[serde(default)]
    pub ticket_price: i32,
    pub description: Option<String>,
}

pub async fn create_event(
    State(s): State<AppState>,
    h: HeaderMap,
    Json(v): Json<NewEvent>,
) -> ApiResult<Value> {
    let admin = authorize(&s, &h)?;
    text(&v.title, 1, 255)?;
    validate_slug(&v.slug)?;
    if v.end_datetime < v.start_datetime || v.ticket_price < 0 {
        return Err(bad("Invalid event time or price"));
    }
    for (t, max) in [
        (&v.venue_name, 255),
        (&v.address, 2000),
        (&v.description, 10000),
    ] {
        if let Some(t) = t {
            text(t, 0, max)?;
        }
    }
    let mut tx = s.pool.begin().await?;
    let id: Option<Uuid> = sqlx::query_scalar(
        "INSERT INTO events(city_id,title,slug,venue_name,address,start_datetime,end_datetime,ticket_price,description) SELECT id,$2,$3,$4,$5,$6,$7,$8,$9 FROM cities WHERE id=$1 AND is_active RETURNING id",
    )
    .bind(v.city_id)
    .bind(v.title)
    .bind(v.slug)
    .bind(v.venue_name)
    .bind(v.address)
    .bind(v.start_datetime)
    .bind(v.end_datetime)
    .bind(v.ticket_price)
    .bind(v.description)
    .fetch_optional(&mut *tx)
    .await?;
    let id = id.ok_or(AppError::NotFound)?;
    audit(&mut tx, &admin.sub, "create_event", id).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Emergency {
    pub city_id: Option<String>,
    pub service_name: String,
    pub category: String,
    pub phone_number: String,
    pub source_url: String,
    #[serde(default)]
    pub is_toll_free: bool,
    #[serde(default)]
    pub is_24_hours: bool,
}

pub async fn create_emergency(
    State(s): State<AppState>,
    h: HeaderMap,
    Json(v): Json<Emergency>,
) -> ApiResult<Value> {
    let admin = authorize(&s, &h)?;
    text(&v.service_name, 1, 150)?;
    text(&v.phone_number, 3, 50)?;
    if sanitize_phone_digits(&v.phone_number).is_none() || https_url(&v.source_url).is_none() {
        return Err(bad("Valid phone and verified HTTPS source required"));
    }
    if !["medis", "keamanan", "pemadam", "sar", "call_center"].contains(&v.category.as_str()) {
        return Err(bad("Invalid emergency category"));
    }
    let mut tx = s.pool.begin().await?;
    if let Some(id) = &v.city_id {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM cities WHERE id=$1 AND is_active)")
                .bind(id)
                .fetch_one(&mut *tx)
                .await?;
        if !exists {
            return Err(AppError::NotFound);
        }
    }
    let id: i32 = sqlx::query_scalar(
        "INSERT INTO emergency_contacts(city_id,service_name,category,phone_number,source_url,verified_at,is_active,is_toll_free,is_24_hours) VALUES($1,$2,$3,$4,$5,now(),true,$6,$7) RETURNING id",
    )
    .bind(v.city_id)
    .bind(v.service_name)
    .bind(v.category)
    .bind(v.phone_number)
    .bind(v.source_url)
    .bind(v.is_toll_free)
    .bind(v.is_24_hours)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO admin_audit(actor,action,target_id,detail) VALUES($1,'create_emergency_contact',$2,$3)")
        .bind(admin.sub)
        .bind(Uuid::new_v4())
        .bind(json!({"contact_id":id}))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id})))
}

async fn audit(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    actor: &str,
    action: &str,
    id: Uuid,
) -> Result<(), AppError> {
    sqlx::query(
        "INSERT INTO admin_audit(actor,action,target_id,detail) VALUES($1,$2,$3,'{}'::jsonb)",
    )
    .bind(actor)
    .bind(action)
    .bind(id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}
