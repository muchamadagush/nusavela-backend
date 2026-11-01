pub mod config;
pub mod error;
pub mod ingest;
pub mod models;
pub mod routes;
pub mod state;
pub mod workers;

use axum::{
    extract::{ConnectInfo, DefaultBodyLimit, Request, State},
    http::{HeaderValue, Method},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use error::{ApiResult, AppError};
use serde_json::{json, Value};
use state::AppState;
use std::time::Duration;
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

pub fn router(s: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(s.config.cors.clone())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([
            axum::http::header::CONTENT_TYPE,
            axum::http::header::AUTHORIZATION,
        ]);
    Router::new()
        .route("/healthz", get(|| async { Json(json!({"status":"ok"})) }))
        .route("/readyz", get(ready))
        .route("/api/v1/coverage", get(routes::catalog::coverage))
        .route("/api/v1/cities", get(routes::catalog::cities))
        .route("/api/v1/cities/nearest", get(routes::catalog::nearest))
        .route("/api/v1/cities/{slug}", get(routes::catalog::city))
        .route("/api/v1/categories", get(routes::catalog::categories))
        .route(
            "/api/v1/kota/{city}/{category}",
            get(routes::catalog::directory),
        )
        .route(
            "/api/v1/kota/{city}/{category}/{place}",
            get(routes::catalog::detail),
        )
        .route("/api/v1/cities/{slug}/events", get(routes::catalog::events))
        .route(
            "/api/v1/cities/{slug}/emergency",
            get(routes::catalog::emergency),
        )
        .route("/api/v1/cities/{slug}/news", get(routes::catalog::news))
        .route(
            "/api/v1/cities/{slug}/weather",
            get(workers::weather::weather),
        )
        .route("/api/v1/submissions", post(routes::community::submit))
        .route("/api/v1/clicks", post(routes::community::click))
        .route("/api/v1/admin/places", post(routes::curation::create_place))
        .route("/api/v1/admin/events", post(routes::curation::create_event))
        .route(
            "/api/v1/admin/emergency",
            post(routes::curation::create_emergency),
        )
        .route("/api/v1/admin/submissions", get(routes::admin::pending))
        .route(
            "/api/v1/admin/submissions/{id}/review",
            post(routes::admin::review),
        )
        .route(
            "/api/v1/admin/places/{id}/sponsorship",
            patch(routes::admin::sponsor),
        )
        .route(
            "/api/v1/admin/places/{id}/affiliate",
            patch(routes::admin::affiliate),
        )
        .route("/api/v1/admin/analytics", get(routes::admin::analytics))
        .fallback(|| async { AppError::NotFound })
        .layer(DefaultBodyLimit::max(16384))
        .layer(middleware::from_fn_with_state(s.clone(), request_policy))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(s)
}
async fn ready(State(s): State<AppState>) -> ApiResult<Value> {
    sqlx::query("SELECT 1").execute(&s.pool).await?;
    Ok(Json(json!({"status":"ready"})))
}
async fn request_policy(State(s): State<AppState>, req: Request, next: Next) -> Response {
    let path = req.uri().path().to_owned();
    if ![Method::GET, Method::HEAD, Method::OPTIONS].contains(req.method()) {
        // Ignore forwarded headers: trust the socket peer only. Enforce real client limits at a trusted reverse proxy.
        let peer = req
            .extensions()
            .get::<ConnectInfo<std::net::SocketAddr>>()
            .map(|x| x.0.ip().to_string())
            .unwrap_or_else(|| "local".into());
        if let Err(e) = rate_limit(&s, &peer).await {
            return e.into_response();
        }
    }
    let response = tokio::time::timeout(Duration::from_secs(20), next.run(req)).await;
    let mut response = match response {
        Ok(v) => v,
        Err(_) => return AppError::Unavailable.into_response(),
    };
    // Normalize extractor rejections to JSON without echoing request bodies or SQL details.
    if response.status().is_client_error()
        && !response
            .headers()
            .get("content-type")
            .is_some_and(|v| v.as_bytes().starts_with(b"application/json"))
    {
        let status = response.status();
        response=(status,Json(json!({"error":{"code":"invalid_request","message":"Invalid request or unsupported method"}}))).into_response();
    }
    response
        .headers_mut()
        .insert("cache-control", HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    if path.starts_with("/api/v1/admin") {
        response
            .headers_mut()
            .insert("pragma", HeaderValue::from_static("no-cache"));
    }
    response
}
async fn rate_limit(s: &AppState, peer: &str) -> Result<(), AppError> {
    let count:i32=sqlx::query_scalar(r#"INSERT INTO request_limits(bucket,window_start,count) VALUES($1,now(),1)
       ON CONFLICT(bucket) DO UPDATE SET
       count=CASE WHEN request_limits.window_start<=now()-interval '60 seconds' THEN 1 ELSE request_limits.count+1 END,
       window_start=CASE WHEN request_limits.window_start<=now()-interval '60 seconds' THEN now() ELSE request_limits.window_start END RETURNING count"#)
        .bind(format!("write:{peer}")).fetch_one(&s.pool).await?;
    if count > 30 {
        return Err(AppError::RateLimited);
    }
    Ok(())
}
