use crate::error::{bad, AppError};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct City {
    pub id: String,
    pub province_id: String,
    pub name: String,
    pub short_name: String,
    pub slug: String,
    pub latitude: f64,
    pub longitude: f64,
    pub tier: i16,
    pub timezone: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Place {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub address: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub phone: Option<String>,
    pub whatsapp: Option<String>,
    pub opening_hours: Option<String>,
    pub is_24_hours: bool,
    pub has_emergency: bool,
    pub is_verified: bool,
    pub is_sponsored: bool,
    pub sponsored_until: Option<DateTime<Utc>>,
    pub sponsored_tier: i16,
    pub distance_meters: Option<f64>,
    pub rating_avg: f64,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub ticket_price_min: Option<i32>,
    pub ticket_price_max: Option<i32>,
    pub affiliate_hotel_url: Option<String>,
    pub affiliate_ticket_url: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct SitemapPlace {
    pub city_slug: String,
    pub category_slug: String,
    pub place_slug: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SitemapQuery {
    pub page: Option<i64>,
}

impl SitemapQuery {
    pub fn validate_page(&self) -> Result<i64, AppError> {
        let page = self.page.unwrap_or(1);
        if !(1..=1000).contains(&page) {
            return Err(bad("Sitemap page must be 1–1000"));
        }
        Ok(page)
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DirectoryQuery {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_meters: Option<f64>,
    pub is_24_hours: Option<bool>,
    pub sort: Option<String>,
    pub page: Option<i64>,
    pub emergency_only: Option<bool>,
}

impl DirectoryQuery {
    pub fn validate(&self) -> Result<(), AppError> {
        match (self.lat, self.lng) {
            (Some(a), Some(b)) => validate_coordinates(a, b)?,
            (None, None) => (),
            _ => return Err(bad("Latitude and longitude must be provided together")),
        }
        let r = self.radius_meters.unwrap_or(25000.);
        if !r.is_finite() || !(100.0..=100000.).contains(&r) {
            return Err(bad("Radius must be 100–100000 meters"));
        }
        if !(1..=10000).contains(&self.page.unwrap_or(1)) {
            return Err(bad("Page must be 1–10000"));
        }
        if self
            .sort
            .as_deref()
            .is_some_and(|v| !["distance", "rating"].contains(&v))
        {
            return Err(bad("Invalid sort"));
        }
        Ok(())
    }
}

pub fn validate_coordinates(lat: f64, lng: f64) -> Result<(), AppError> {
    if !lat.is_finite()
        || !lng.is_finite()
        || !(-90.0..=90.).contains(&lat)
        || !(-180.0..=180.).contains(&lng)
    {
        return Err(bad("Invalid coordinates"));
    }
    Ok(())
}

pub fn text(value: &str, min: usize, max: usize) -> Result<(), AppError> {
    let len = value.trim().chars().count();
    if len < min || len > max {
        return Err(bad("Invalid text length"));
    }
    Ok(())
}

pub fn validate_slug(s: &str) -> Result<(), AppError> {
    text(s, 1, 300)?;
    if !s
        .bytes()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
    {
        return Err(bad(
            "Slug must contain only lowercase ASCII letters, digits and hyphens",
        ));
    }
    Ok(())
}

pub fn sanitize_phone_digits(value: &str) -> Option<String> {
    if !value
        .chars()
        .all(|c| c.is_ascii_digit() || "+ -()".contains(c))
    {
        return None;
    }
    let digits: String = value.chars().filter(char::is_ascii_digit).collect();
    (3..=15).contains(&digits.len()).then_some(digits)
}

pub fn https_url(value: &str) -> Option<String> {
    let u = url::Url::parse(value).ok()?;
    (u.scheme() == "https"
        && u.host_str().is_some()
        && u.username().is_empty()
        && u.password().is_none())
    .then(|| u.to_string())
}
