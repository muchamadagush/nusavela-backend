use crate::{models::https_url, state::AppState};
use chrono::{Duration, Utc};
use scraper::Html;

pub fn summary(input: &str) -> String {
    let html = Html::parse_fragment(input);
    html.root_element()
        .text()
        .collect::<Vec<_>>()
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(160)
        .collect()
}
async fn fetch_feed(s: &AppState, province: &str, feed_url: &str) -> anyhow::Result<()> {
    let u = url::Url::parse(feed_url)?;
    if https_url(feed_url).is_none() {
        anyhow::bail!("RSS must be HTTPS");
    }
    let host = u
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("RSS host missing"))?;
    if u.port_or_known_default() != Some(443) {
        anyhow::bail!("RSS port not allowed");
    }
    let addresses: Vec<_> = tokio::net::lookup_host((host, 443)).await?.collect();
    if addresses.is_empty() || addresses.iter().any(|a| !public_ip(a.ip())) {
        anyhow::bail!("RSS host resolves to a non-public address");
    }
    // Pin the validated DNS result; redirects disabled, preventing DNS rebinding/redirect SSRF.
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .resolve_to_addrs(host, &addresses)
        .timeout(std::time::Duration::from_secs(15))
        .build()?;
    let mut response = client.get(u.clone()).send().await?.error_for_status()?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() + chunk.len() > 2 * 1024 * 1024 {
            anyhow::bail!("RSS feed too large");
        }
        bytes.extend_from_slice(&chunk);
    }
    let feed = feed_rs::parser::parse(bytes.as_slice())?;
    let source = feed
        .title
        .map(|x| summary(&x.content).chars().take(100).collect::<String>())
        .filter(|x| !x.is_empty())
        .unwrap_or_else(|| host.to_owned());
    for item in feed.entries.into_iter().take(100) {
        let Some(link) = item.links.iter().find_map(|l| https_url(&l.href)) else {
            continue;
        };
        let title = item.title.map(|x| summary(&x.content)).unwrap_or_default();
        if title.trim().is_empty() {
            continue;
        }
        let snippet = item.summary.map(|x| summary(&x.content));
        let published = item.published.or(item.updated).unwrap_or_else(Utc::now);
        if published > Utc::now() + Duration::hours(24)
            || published < Utc::now() - Duration::days(30)
        {
            continue;
        }
        // Deliberately keep ambiguous news at province scope, rather than guessing city by substring.
        sqlx::query("INSERT INTO news_feeds(province_id,title,summary,source_name,source_url,published_at) VALUES($1,$2,$3,$4,$5,$6) ON CONFLICT(source_url) DO UPDATE SET title=EXCLUDED.title,summary=EXCLUDED.summary,source_name=EXCLUDED.source_name")
            .bind(province).bind(title).bind(snippet).bind(&source).bind(link).bind(published).execute(&s.pool).await?;
    }
    Ok(())
}
pub fn public_ip(ip: std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v) => {
            let n = v.octets();
            !(v.is_private()
                || v.is_loopback()
                || v.is_link_local()
                || v.is_unspecified()
                || v.is_broadcast()
                || v.is_multicast()
                || n[0] == 0
                || n[0] >= 240
                || n[0] == 100 && (64..=127).contains(&n[1])
                || n[0] == 198 && (n[1] == 18 || n[1] == 19)
                || n[0] == 192 && n[1] == 0
                || n[0] == 198 && n[1] == 51
                || n[0] == 203 && n[1] == 0 && n[2] == 113)
        }
        std::net::IpAddr::V6(v) => v
            .to_ipv4_mapped()
            .map(|v| public_ip(v.into()))
            .unwrap_or_else(|| {
                let a = v.segments();
                a[0] & 0xe000 == 0x2000 && !(a[0] == 0x2001 && (a[1] == 0x0db8 || a[1] < 0x0200))
            }),
    }
}
pub async fn cycle(s: &AppState) -> anyhow::Result<()> {
    let mut lock = s.pool.acquire().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock(8125601)")
        .fetch_one(&mut *lock)
        .await?;
    if !acquired {
        return Ok(());
    }
    let result = async {
        let rows: Vec<(String, Vec<String>)> = sqlx::query_as(
            "SELECT id,rss_news_urls FROM provinces WHERE cardinality(rss_news_urls)>0",
        )
        .fetch_all(&s.pool)
        .await?;
        for (province, urls) in rows {
            for url in urls {
                if let Err(e) = fetch_feed(s, &province, &url).await {
                    tracing::warn!(error=%e,province=%province,"RSS ingestion failed");
                }
            }
        }
        sqlx::query("DELETE FROM news_feeds WHERE published_at<now()-interval '30 days'")
            .execute(&s.pool)
            .await?;
        sqlx::query("DELETE FROM request_limits WHERE window_start<now()-interval '2 minutes'")
            .execute(&s.pool)
            .await?;
        Ok::<_, anyhow::Error>(())
    }
    .await;
    sqlx::query("SELECT pg_advisory_unlock(8125601)")
        .execute(&mut *lock)
        .await?;
    result
}
pub async fn run(s: AppState) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(7200));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        timer.tick().await;
        if let Err(e) = cycle(&s).await {
            tracing::warn!(error=%e,"RSS cycle failed");
        }
    }
}
