use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    BadRequest(String),
    #[error("Resource not found")]
    NotFound,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Conflict")]
    Conflict,
    #[error("Service unavailable")]
    Unavailable,
    #[error("Too many requests")]
    RateLimited,
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    #[error("Upstream error")]
    Upstream(#[from] reqwest::Error),
}
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::BadRequest(m) => (StatusCode::BAD_REQUEST, "invalid_request", m.as_str()),
            Self::NotFound => (StatusCode::NOT_FOUND, "not_found", "Resource not found"),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized", "Unauthorized"),
            Self::Conflict => (
                StatusCode::CONFLICT,
                "conflict",
                "Resource already reviewed or changed",
            ),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "Service unavailable",
            ),
            Self::RateLimited => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many requests",
            ),
            Self::Database(sqlx::Error::Database(e)) if e.is_unique_violation() => {
                (StatusCode::CONFLICT, "conflict", "Resource already exists")
            }
            Self::Database(sqlx::Error::Database(e))
                if e.is_foreign_key_violation() || e.is_check_violation() =>
            {
                (
                    StatusCode::BAD_REQUEST,
                    "invalid_request",
                    "Invalid relation or value",
                )
            }
            Self::Database(_) | Self::Upstream(_) => {
                tracing::error!(error = %self, "request dependency failure");
                (
                    StatusCode::SERVICE_UNAVAILABLE,
                    "dependency_unavailable",
                    "Service temporarily unavailable",
                )
            }
        };
        let mut response = (
            status,
            Json(json!({ "error": { "code": code, "message": message } })),
        )
            .into_response();
        response.headers_mut().insert(
            "cache-control",
            axum::http::HeaderValue::from_static("no-store"),
        );
        response
    }
}
pub type ApiResult<T> = Result<Json<T>, AppError>;
pub fn bad(message: &str) -> AppError {
    AppError::BadRequest(message.to_owned())
}
