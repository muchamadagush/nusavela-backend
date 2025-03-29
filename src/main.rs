use nusavela_backend::{config::Config, state::AppState};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nusavela_backend=info,tower_http=info".into()),
        )
        .init();
    let config = Config::from_env()?;
    let bind = config.bind;
    let max_connections: u32 = std::env::var("DATABASE_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".into())
        .parse()?;
    anyhow::ensure!(
        (1..=100).contains(&max_connections),
        "DATABASE_MAX_CONNECTIONS must be 1–100"
    );
    anyhow::ensure!(
        !config.rss_enabled || max_connections >= 2,
        "RSS requires at least 2 database connections"
    );
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(Duration::from_secs(5))
        .after_connect(|conn, _| {
            Box::pin(async move {
                sqlx::query("SET statement_timeout = '10s'")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    let state = AppState::new(pool.clone(), config)?;
    let rss = state
        .config
        .rss_enabled
        .then(|| tokio::spawn(nusavela_backend::workers::rss::run(state.clone())));
    let maintenance = tokio::spawn(nusavela_backend::workers::maintenance::run(pool.clone()));
    let listener = tokio::net::TcpListener::bind(bind).await?;
    tracing::info!(address=%bind,"NusaVela API listening");
    axum::serve(
        listener,
        nusavela_backend::router(state)
            .into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    if let Some(worker) = rss {
        worker.abort();
    }
    maintenance.abort();
    pool.close().await;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! { _=tokio::signal::ctrl_c()=>(), _=terminate.recv()=>() }
            }
            Err(_) => {
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
