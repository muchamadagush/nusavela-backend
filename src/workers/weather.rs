use crate::{
    error::{ApiResult, AppError},
    routes::catalog::find_city,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};
use std::time::{Duration, Instant};
pub async fn weather(State(s): State<AppState>, Path(slug): Path<String>) -> ApiResult<Value> {
    let c = find_city(&s.pool, &slug).await?;
    let key = s.config.weather_key.as_ref().ok_or(AppError::Unavailable)?;
    // Serialize cold fetches to prevent an upstream stampede; cache is bounded by city count.
    let mut entries = s.weather.entries.lock().await;
    if let Some((time, value)) = entries.get(&c.id) {
        if time.elapsed() < Duration::from_secs(1800) {
            return Ok(Json(value.clone()));
        }
    }
    entries.retain(|_, (t, _)| t.elapsed() < Duration::from_secs(1800));
    let response = s
        .client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&[
            ("lat", c.latitude.to_string()),
            ("lon", c.longitude.to_string()),
            ("appid", key.clone()),
            ("units", "metric".into()),
            ("lang", "id".into()),
        ])
        .send()
        .await?
        .error_for_status()?;
    // Do not return URLs containing API keys, or the raw provider response.
    let v: Value = response.json().await?;
    let value = json!({"city_id":c.id,"temperature_c":v["main"]["temp"],"description":v["weather"][0]["description"],"observed_at":v["dt"],"attribution":{"name":"OpenWeather","url":"https://openweathermap.org/"}});
    entries.insert(c.id, (Instant::now(), value.clone()));
    Ok(Json(value))
}
