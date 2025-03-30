use crate::{
    auth::{hash_password, issue_jwt, verify_password},
    error::{bad, ApiResult, AppError},
    models::text,
    routes::admin::authorize,
    state::AppState,
};
use axum::{
    extract::State,
    http::HeaderMap,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, FromRow)]
pub struct UserPublic {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub full_name: String,
    pub role: String,
    pub is_active: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, FromRow)]
struct UserRecord {
    id: Uuid,
    email: String,
    username: String,
    full_name: String,
    password_hash: String,
    role: String,
    is_active: bool,
    last_login_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

impl From<UserRecord> for UserPublic {
    fn from(r: UserRecord) -> Self {
        Self {
            id: r.id,
            email: r.email,
            username: r.username,
            full_name: r.full_name,
            role: r.role,
            is_active: r.is_active,
            last_login_at: r.last_login_at,
            created_at: r.created_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoginRequest {
    pub identifier: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub full_name: String,
    pub password: String,
    pub role: Option<String>,
}

impl CreateUserRequest {
    fn validate(&self) -> Result<(String, String, String), AppError> {
        let email = self.email.trim().to_lowercase();
        let username = self.username.trim().to_lowercase();
        text(&email, 5, 255)?;
        if !email.contains('@') || !email.contains('.') || email.contains(' ') {
            return Err(bad("Valid email address required"));
        }
        text(&username, 3, 100)?;
        if !username
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        {
            return Err(bad(
                "Username may only contain lowercase letters, numbers, dots, underscores, and hyphens",
            ));
        }
        text(&self.full_name, 2, 150)?;
        let role = self.role.as_deref().unwrap_or("admin").to_owned();
        if !["admin", "operator"].contains(&role.as_str()) {
            return Err(bad("Role must be admin or operator"));
        }
        Ok((email, username, role))
    }
}

pub async fn status(State(s): State<AppState>) -> ApiResult<Value> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE is_active")
        .fetch_one(&s.pool)
        .await?;
    Ok(Json(json!({
        "needs_initial_setup": count == 0,
        "active_users": count
    })))
}

pub async fn login(State(s): State<AppState>, Json(v): Json<LoginRequest>) -> ApiResult<Value> {
    let ident = v.identifier.trim().to_lowercase();
    text(&ident, 3, 255)?;
    text(&v.password, 1, 128)?;

    let user: Option<UserRecord> = sqlx::query_as(
        "SELECT id, email, username, full_name, password_hash, role, is_active, last_login_at, created_at FROM users WHERE (email = $1 OR username = $1) AND is_active",
    )
    .bind(&ident)
    .fetch_optional(&s.pool)
    .await?;

    let Some(user) = user else {
        return Err(AppError::Unauthorized);
    };
    if !verify_password(&v.password, &user.password_hash) {
        return Err(AppError::Unauthorized);
    }

    sqlx::query("UPDATE users SET last_login_at = now(), updated_at = now() WHERE id = $1")
        .bind(user.id)
        .execute(&s.pool)
        .await?;

    let (token, expires_in) = issue_jwt(&s, &user.username, &user.role, None)?;
    let profile = UserPublic::from(user);

    Ok(Json(json!({
        "token": token,
        "expires_in": expires_in,
        "user": profile
    })))
}

pub async fn setup_initial_admin(
    State(s): State<AppState>,
    Json(v): Json<CreateUserRequest>,
) -> ApiResult<Value> {
    let (email, username, _) = v.validate()?;
    let password_hash = hash_password(&v.password)?;

    let mut tx = s.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(8125609)")
        .execute(&mut *tx)
        .await?;
    let existing_count: i64 = sqlx::query_scalar("SELECT count(*) FROM users")
        .fetch_one(&mut *tx)
        .await?;
    if existing_count > 0 {
        return Err(AppError::Conflict);
    }

    let user: UserPublic = sqlx::query_as(
        "INSERT INTO users(email, username, full_name, password_hash, role, last_login_at) VALUES($1, $2, $3, $4, 'admin', now()) RETURNING id, email, username, full_name, role, is_active, last_login_at, created_at",
    )
    .bind(&email)
    .bind(&username)
    .bind(v.full_name.trim())
    .bind(password_hash)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO admin_audit(actor, action, target_id, detail) VALUES($1, 'setup_initial_admin', $2, $3)")
        .bind(&username)
        .bind(user.id)
        .bind(json!({"email": email, "username": username, "role": "admin"}))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    let (token, expires_in) = issue_jwt(&s, &user.username, &user.role, None)?;
    Ok(Json(json!({
        "token": token,
        "expires_in": expires_in,
        "user": user
    })))
}

pub async fn me(State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Value> {
    let claims = authorize(&s, &headers)?;
    let user: Option<UserPublic> = sqlx::query_as(
        "SELECT id, email, username, full_name, role, is_active, last_login_at, created_at FROM users WHERE username = $1 AND is_active",
    )
    .bind(&claims.sub)
    .fetch_optional(&s.pool)
    .await?;

    if let Some(u) = user {
        Ok(Json(json!({"user": u})))
    } else {
        Ok(Json(json!({
            "user": {
                "id": Uuid::nil(),
                "email": format!("{}@nusavela.com", claims.sub),
                "username": claims.sub,
                "full_name": claims.sub,
                "role": claims.role,
                "is_active": true,
                "last_login_at": null,
                "created_at": Utc::now()
            }
        })))
    }
}

pub async fn list_users(State(s): State<AppState>, headers: HeaderMap) -> ApiResult<Vec<UserPublic>> {
    authorize(&s, &headers)?;
    let users = sqlx::query_as::<_, UserPublic>(
        "SELECT id, email, username, full_name, role, is_active, last_login_at, created_at FROM users ORDER BY created_at ASC",
    )
    .fetch_all(&s.pool)
    .await?;
    Ok(Json(users))
}

pub async fn create_user(
    State(s): State<AppState>,
    headers: HeaderMap,
    Json(v): Json<CreateUserRequest>,
) -> ApiResult<UserPublic> {
    let admin = authorize(&s, &headers)?;
    if admin.role != "admin" {
        return Err(AppError::Unauthorized);
    }
    let (email, username, role) = v.validate()?;
    let password_hash = hash_password(&v.password)?;

    let mut tx = s.pool.begin().await?;
    let user: UserPublic = sqlx::query_as(
        "INSERT INTO users(email, username, full_name, password_hash, role) VALUES($1, $2, $3, $4, $5) RETURNING id, email, username, full_name, role, is_active, last_login_at, created_at",
    )
    .bind(&email)
    .bind(&username)
    .bind(v.full_name.trim())
    .bind(password_hash)
    .bind(&role)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO admin_audit(actor, action, target_id, detail) VALUES($1, 'create_user', $2, $3)")
        .bind(&admin.sub)
        .bind(user.id)
        .bind(json!({"email": email, "username": username, "role": role}))
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(user))
}
