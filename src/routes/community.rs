use crate::{
    error::{bad, ApiResult, AppError},
    models::{https_url, text},
    state::AppState,
};
use axum::{extract::State, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub city_id: String,
    pub place_id: Option<Uuid>,
    pub submission_type: String,
    pub submitter_name: String,
    pub submitter_contact: String,
    pub payload: Value,
}
impl Submission {
    pub fn validate(&self) -> Result<(), AppError> {
        text(&self.city_id, 1, 10)?;
        text(&self.submitter_name, 1, 100)?;
        text(&self.submitter_contact, 3, 100)?;
        if !["correction", "business_claim", "new_place", "new_event"]
            .contains(&self.submission_type.as_str())
        {
            return Err(bad("Invalid submission type"));
        }
        if ["correction", "business_claim"].contains(&self.submission_type.as_str())
            && self.place_id.is_none()
        {
            return Err(bad("Place required for corrections and claims"));
        }
        if !self.payload.is_object() {
            return Err(bad("Payload must be an object"));
        }
        let message = self
            .payload
            .get("message")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("Message required"))?;
        text(message, 10, 3000)?;
        if self.payload.as_object().is_some_and(|p| p.len() != 1) {
            return Err(bad(
                "Only the message field is accepted in the submission payload",
            ));
        }
        Ok(())
    }
}
pub async fn submit(State(s): State<AppState>, Json(v): Json<Submission>) -> ApiResult<Value> {
    v.validate()?;
    let id: Option<Uuid> = sqlx::query_scalar(r#"INSERT INTO community_submissions(city_id,place_id,submission_type,submitter_name,submitter_contact,payload)
      SELECT c.id,$2,$3,$4,$5,$6 FROM cities c WHERE c.id=$1 AND c.is_active
      AND ($2::uuid IS NULL OR EXISTS(SELECT 1 FROM places WHERE id=$2 AND city_id=c.id AND is_active)) RETURNING id"#)
        .bind(&v.city_id).bind(v.place_id).bind(&v.submission_type).bind(v.submitter_name.trim()).bind(v.submitter_contact.trim()).bind(v.payload)
        .fetch_optional(&s.pool).await?;
    Ok(Json(json!({"id":id.ok_or(AppError::NotFound)?})))
}
#[derive(sqlx::FromRow)]
struct ClickTargets {
    affiliate_hotel_url: Option<String>,
    affiliate_ticket_url: Option<String>,
    whatsapp: Option<String>,
    phone: Option<String>,
    category: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Click {
    pub city_id: String,
    pub place_id: Uuid,
    pub click_type: String,
}
pub async fn click(State(s): State<AppState>, Json(v): Json<Click>) -> ApiResult<Value> {
    let row: Option<ClickTargets> = sqlx::query_as("SELECT p.affiliate_hotel_url,p.affiliate_ticket_url,p.whatsapp,p.phone,cat.slug AS category FROM places p JOIN cities c ON p.city_id=c.id JOIN categories cat ON cat.id=p.category_id WHERE p.id=$1 AND p.city_id=$2 AND p.is_active AND c.is_active")
        .bind(v.place_id).bind(&v.city_id).fetch_optional(&s.pool).await?;
    let ClickTargets {
        affiliate_hotel_url: hotel,
        affiliate_ticket_url: ticket,
        whatsapp,
        phone,
        category,
    } = row.ok_or(AppError::NotFound)?;
    let target = match v.click_type.as_str() {
        "affiliate_hotel" | "affiliate_ticket"
            if ["rumah-sakit", "layanan-publik"].contains(&category.as_str()) =>
        {
            return Err(bad(
                "Affiliate tracking is disabled on public safety directories",
            ))
        }
        "affiliate_hotel" => hotel.and_then(|x| https_url(&x)),
        "affiliate_ticket" => ticket.and_then(|x| https_url(&x)),
        "whatsapp_lead" => whatsapp
            .and_then(|n| digits(&n))
            .map(|n| format!("https://wa.me/{n}")),
        "phone_call" => phone.and_then(|n| digits(&n)).map(|n| format!("tel:+{n}")),
        _ => return Err(bad("Invalid click type")),
    }
    .ok_or_else(|| bad("No valid destination configured"))?;
    sqlx::query("INSERT INTO monetization_clicks(city_id,place_id,click_type,target_url) VALUES($1,$2,$3,$4)")
        .bind(v.city_id).bind(v.place_id).bind(v.click_type).bind(&target).execute(&s.pool).await?;
    Ok(Json(json!({"ok":true,"target_url":target})))
}
fn digits(value: &str) -> Option<String> {
    if !value
        .chars()
        .all(|c| c.is_ascii_digit() || "+ -()".contains(c))
    {
        return None;
    }
    let n: String = value.chars().filter(char::is_ascii_digit).collect();
    (5..=15).contains(&n.len()).then_some(n)
}
