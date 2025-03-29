use sqlx::PgPool;
pub async fn run(pool: PgPool) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(60));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        timer.tick().await;
        if let Err(e) =
            sqlx::query("DELETE FROM request_limits WHERE window_start<now()-interval '2 minutes'")
                .execute(&pool)
                .await
        {
            tracing::warn!(error=%e,"rate limit cleanup failed");
        }
    }
}
