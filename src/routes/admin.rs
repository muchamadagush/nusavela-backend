use crate::{
    error::{bad, ApiResult, AppError},
    models::{https_url, text},
    state::AppState,
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: u64,
    pub nbf: u64,
    pub iss: String,
    pub aud: String,
}
pub fn authorize(s: &AppState, headers: &HeaderMap) -> Result<Claims, AppError> {
    let secret = s
        .config
        .admin_secret
        .as_ref()
        .ok_or(AppError::Unauthorized)?;
    let token = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized)?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[&s.config.admin_issuer]);
    validation.set_audience(&[&s.config.admin_audience]);
    validation.validate_nbf = true;
    validation.leeway = 10;
    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AppError::Unauthorized)?
    .claims;
    if claims.role != "admin" || claims.sub.trim().is_empty() {
        return Err(AppError::Unauthorized);
    }
    Ok(claims)
}
pub async fn pending(State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Value> {
    authorize(&s, &headers)?;
    let rows: Vec<Value> = sqlx::query_scalar("SELECT to_jsonb(s) FROM community_submissions s WHERE status='pending' ORDER BY created_at,id LIMIT 100").fetch_all(&s.pool).await?;
    Ok(Json(json!(rows)))
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacePatch {
    pub description: Option<String>,
    pub opening_hours: Option<String>,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub is_24_hours: Option<bool>,
    pub has_emergency: Option<bool>,
    pub is_verified: Option<bool>,
}
impl PlacePatch {
    fn validate(&self) -> Result<(), AppError> {
        for (v, max) in [
            (&self.description, 10000),
            (&self.opening_hours, 255),
            (&self.phone, 50),
            (&self.whatsapp, 50),
        ] {
            if let Some(t) = v {
                text(t, 0, max)?;
            }
        }
        Ok(())
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub status: String,
    pub notes: String,
    pub patch: Option<PlacePatch>,
    pub created_entity_id: Option<Uuid>,
}
pub async fn review(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(v): Json<Review>,
) -> ApiResult<Value> {
    let admin = authorize(&s, &headers)?;
    text(&v.notes, 3, 2000)?;
    if !["approved", "rejected"].contains(&v.status.as_str()) {
        return Err(bad("Review must approve or reject"));
    }
    if v.status == "rejected" && v.patch.is_some() {
        return Err(bad("Rejected submissions cannot modify a place"));
    }
    if let Some(p) = &v.patch {
        p.validate()?;
    }
    let mut tx = s.pool.begin().await?;
    let row: Option<(Option<Uuid>,String,String,String)> = sqlx::query_as("SELECT place_id,status,submission_type,city_id FROM community_submissions WHERE id=$1 FOR UPDATE").bind(id).fetch_optional(&mut *tx).await?;
    let (place_id, status, kind, city_id) = row.ok_or(AppError::NotFound)?;
    if status != "pending" {
        return Err(AppError::Conflict);
    }
    if v.status == "approved" && ["new_place", "new_event"].contains(&kind.as_str()) {
        if v.patch.is_some() {
            return Err(bad(
                "Use a curated entity ID for new place/event submissions",
            ));
        }
        let entity = v
            .created_entity_id
            .ok_or_else(|| bad("Create the curated entity first and provide its ID"))?;
        let sql = if kind == "new_place" {
            "SELECT EXISTS(SELECT 1 FROM places WHERE id=$1 AND city_id=$2 AND is_active)"
        } else {
            "SELECT EXISTS(SELECT 1 FROM events WHERE id=$1 AND city_id=$2 AND is_active)"
        };
        let exists: bool = sqlx::query_scalar(sql)
            .bind(entity)
            .bind(city_id)
            .fetch_one(&mut *tx)
            .await?;
        if !exists {
            return Err(bad("Curated entity must belong to the submission city"));
        }
    }
    if v.status == "approved"
        && ["correction", "business_claim"].contains(&kind.as_str())
        && v.patch.is_none()
    {
        return Err(bad(
            "An approved correction or claim requires an explicit reviewed patch",
        ));
    }
    let reviewed_patch = json!(&v.patch);
    if let Some(p) = v.patch {
        let place = place_id.ok_or_else(|| bad("Submission has no place"))?;
        let changed=sqlx::query("UPDATE places SET description=COALESCE($2,description),opening_hours=COALESCE($3,opening_hours),phone=COALESCE($4,phone),whatsapp=COALESCE($5,whatsapp),is_24_hours=COALESCE($6,is_24_hours),has_emergency=COALESCE($7,has_emergency),is_verified=COALESCE($8,is_verified),source='admin',last_verified_at=now(),updated_at=now() WHERE id=$1")
          .bind(place).bind(p.description).bind(p.opening_hours).bind(p.phone).bind(p.whatsapp).bind(p.is_24_hours).bind(p.has_emergency).bind(p.is_verified).execute(&mut *tx).await?;
        if changed.rows_affected() != 1 {
            return Err(AppError::NotFound);
        }
    }
    sqlx::query("UPDATE community_submissions SET status=$2,admin_notes=$3,reviewed_by=$4,reviewed_at=now() WHERE id=$1").bind(id).bind(&v.status).bind(&v.notes).bind(&admin.sub).execute(&mut *tx).await?;
    sqlx::query("INSERT INTO admin_audit(actor,action,target_id,detail) VALUES($1,'review_submission',$2,$3)").bind(admin.sub).bind(id).bind(json!({"status":v.status,"notes":v.notes,"patch":reviewed_patch,"created_entity_id":v.created_entity_id})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(json!({"id":id,"status":v.status})))
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sponsorship {
    pub tier: i16,
    pub until: Option<DateTime<Utc>>,
    pub payment_reference: String,
}
pub async fn sponsor(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(v): Json<Sponsorship>,
) -> ApiResult<Value> {
    let admin = authorize(&s, &headers)?;
    text(&v.payment_reference, 3, 200)?;
    if !(0..=2).contains(&v.tier) {
        return Err(bad("Tier must be 0–2"));
    }
    if v.tier > 0
        && !v
            .until
            .is_some_and(|d| d > Utc::now() && d <= Utc::now() + chrono::Duration::days(366))
    {
        return Err(bad("Sponsor expiry must be within the next 366 days"));
    }
    let mut tx = s.pool.begin().await?;
    let changed=sqlx::query("UPDATE places SET is_sponsored=$2,sponsored_tier=$3,sponsored_until=$4,updated_at=now() WHERE id=$1 AND is_active")
        .bind(id).bind(v.tier>0).bind(v.tier).bind(if v.tier==0 {None} else {v.until}).execute(&mut *tx).await?;
    if changed.rows_affected() != 1 {
        return Err(AppError::NotFound);
    }
    sqlx::query(
        "INSERT INTO admin_audit(actor,action,target_id,detail) VALUES($1,'set_sponsorship',$2,$3)",
    )
    .bind(admin.sub)
    .bind(id)
    .bind(json!({"tier":v.tier,"until":v.until,"payment_reference":v.payment_reference}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Affiliate {
    pub hotel_url: Option<String>,
    pub ticket_url: Option<String>,
}
pub async fn affiliate(
    State(s): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(v): Json<Affiliate>,
) -> ApiResult<Value> {
    let admin = authorize(&s, &headers)?;
    for u in [&v.hotel_url, &v.ticket_url].into_iter().flatten() {
        if https_url(u).is_none() {
            return Err(bad(
                "Affiliate links must be absolute HTTPS URLs without credentials",
            ));
        }
    }
    let mut tx = s.pool.begin().await?;
    let n=sqlx::query("UPDATE places SET affiliate_hotel_url=$2,affiliate_ticket_url=$3,updated_at=now() WHERE id=$1 AND is_active").bind(id).bind(&v.hotel_url).bind(&v.ticket_url).execute(&mut *tx).await?;
    if n.rows_affected() != 1 {
        return Err(AppError::NotFound);
    }
    sqlx::query(
        "INSERT INTO admin_audit(actor,action,target_id,detail) VALUES($1,'set_affiliate',$2,$3)",
    )
    .bind(admin.sub)
    .bind(id)
    .bind(json!({"hotel_url":v.hotel_url,"ticket_url":v.ticket_url}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(json!({"ok":true})))
}
pub async fn analytics(State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Value> {
    authorize(&s, &headers)?;
    let rows: Vec<Value>=sqlx::query_scalar("SELECT jsonb_build_object('city_id',city_id,'click_type',click_type,'clicks',count(*)) FROM monetization_clicks WHERE clicked_at>=now()-interval '30 days' GROUP BY city_id,click_type ORDER BY count(*) DESC").fetch_all(&s.pool).await?;
    Ok(Json(json!(rows)))
}
