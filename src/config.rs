use anyhow::{bail, Context};
use axum::http::HeaderValue;
use std::{env, net::SocketAddr};

pub const DEFAULT_DEV_JWT_SECRET: &str = "nusavela-local-dev-jwt-secret-key-at-least-32-bytes";

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub bind: SocketAddr,
    pub cors: Vec<HeaderValue>,
    pub admin_secret: Option<String>,
    pub admin_issuer: String,
    pub admin_audience: String,
    pub weather_key: Option<String>,
    pub rss_enabled: bool,
}
impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let secret = match env::var("ADMIN_JWT_SECRET").ok().filter(|s| !s.trim().is_empty()) {
            Some(s) => {
                if s.len() < 32 {
                    bail!("ADMIN_JWT_SECRET must contain at least 32 bytes");
                }
                Some(s)
            }
            None => Some(DEFAULT_DEV_JWT_SECRET.to_owned()),
        };
        let origins = env::var("CORS_ORIGINS")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "http://localhost:3000,http://127.0.0.1:3000".into());
        let mut cors = Vec::new();
        for origin in origins.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let u = url::Url::parse(origin).context("CORS origin must be an absolute URL")?;
            if !["http", "https"].contains(&u.scheme())
                || u.host_str().is_none()
                || u.origin().ascii_serialization() != origin
            {
                bail!("CORS entries must be exact origins without paths, credentials or wildcards");
            }
            cors.push(origin.parse()?);
        }
        Ok(Self {
            database_url: env::var("DATABASE_URL").context("DATABASE_URL required")?,
            bind: env::var("BIND_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".into())
                .parse()?,
            cors,
            admin_secret: secret,
            admin_issuer: env::var("ADMIN_JWT_ISSUER")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "nusavela-admin".into()),
            admin_audience: env::var("ADMIN_JWT_AUDIENCE")
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "nusavela-api".into()),
            weather_key: env::var("OPENWEATHER_API_KEY")
                .ok()
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty()),
            rss_enabled: env::var("RSS_ENABLED").unwrap_or_default() == "true",
        })
    }
}
