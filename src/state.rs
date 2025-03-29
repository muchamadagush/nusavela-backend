use crate::config::Config;
use std::sync::Arc;
#[derive(Clone)]
pub struct AppState {
    pub pool: sqlx::PgPool,
    pub config: Arc<Config>,
    pub client: reqwest::Client,
    pub weather: Arc<WeatherCache>,
}
impl AppState {
    pub fn new(pool: sqlx::PgPool, config: Config) -> anyhow::Result<Self> {
        Ok(Self {
            pool,
            config: Arc::new(config),
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(10))
                .redirect(reqwest::redirect::Policy::none())
                .user_agent("NusaVela/0.1 (+https://nusavela.com)")
                .build()?,
            weather: Arc::new(WeatherCache::default()),
        })
    }
}

#[derive(Default)]
pub struct WeatherCache {
    pub(crate) entries: tokio::sync::Mutex<
        std::collections::HashMap<String, (std::time::Instant, serde_json::Value)>,
    >,
}
