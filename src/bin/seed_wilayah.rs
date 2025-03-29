use clap::Parser;
use nusavela_backend::models::validate_coordinates;
use serde::Deserialize;
use std::{collections::HashSet, path::PathBuf};

#[derive(Parser)]
#[command(about = "Import a versioned, coordinate-complete administrative snapshot atomically")]
struct Args {
    #[arg(long)]
    file: PathBuf,
    /// Validate coverage, IDs, coordinates and metadata without connecting to a database.
    #[arg(long)]
    check_only: bool,
    #[arg(long, default_value_t = 38)]
    expected_provinces: usize,
    #[arg(long, default_value_t = 514)]
    expected_cities: usize,
    /// Optional emsifa-compatible API base; codes/names must match the snapshot exactly.
    #[arg(long)]
    verify_emsifa_base: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Dataset {
    source: String,
    version: String,
    provinces: Vec<Province>,
    cities: Vec<City>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Province {
    id: String,
    name: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct City {
    id: String,
    province_id: String,
    name: String,
    short_name: String,
    latitude: f64,
    longitude: f64,
    timezone: String,
    #[serde(default = "tier")]
    tier: i16,
}
fn tier() -> i16 {
    2
}
#[derive(Deserialize)]
struct Remote {
    id: String,
    name: String,
    #[serde(default)]
    province_id: String,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let a = Args::parse();
    let data: Dataset = serde_json::from_slice(&tokio::fs::read(a.file).await?)?;
    anyhow::ensure!(
        !data.source.trim().is_empty() && !data.version.trim().is_empty(),
        "Source and version required"
    );
    anyhow::ensure!(
        data.provinces.len() == a.expected_provinces && data.cities.len() == a.expected_cities,
        "Dataset coverage mismatch; expected {}/{} provinces/cities, received {}/{}",
        a.expected_provinces,
        a.expected_cities,
        data.provinces.len(),
        data.cities.len()
    );
    let mut provinces = HashSet::new();
    let mut cities = HashSet::new();
    let mut slugs = HashSet::new();
    for p in &data.provinces {
        anyhow::ensure!(
            !p.id.is_empty()
                && p.id.len() <= 10
                && !p.name.is_empty()
                && p.name.chars().count() <= 100
                && provinces.insert(p.id.clone()),
            "Invalid/duplicate province"
        );
    }
    for c in &data.cities {
        validate_coordinates(c.latitude, c.longitude)?;
        anyhow::ensure!(
            provinces.contains(&c.province_id)
                && cities.insert(c.id.clone())
                && slugs.insert(slug(&c.name)),
            "Invalid city reference or duplicate city/slug"
        );
        anyhow::ensure!(
            !c.id.is_empty()
                && c.id.len() <= 10
                && [1, 2].contains(&c.tier)
                && ["Asia/Jakarta", "Asia/Makassar", "Asia/Jayapura"]
                    .contains(&c.timezone.as_str()),
            "Invalid city metadata"
        );
        anyhow::ensure!(
            !c.name.is_empty()
                && c.name.chars().count() <= 150
                && !c.short_name.is_empty()
                && c.short_name.chars().count() <= 100,
            "Invalid city name"
        );
    }
    if let Some(base) = a.verify_emsifa_base {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()?;
        let remote: Vec<Remote> = client
            .get(format!("{}/provinces.json", base.trim_end_matches('/')))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        anyhow::ensure!(
            remote.len() == data.provinces.len(),
            "Upstream province coverage differs; verify administrative vintage"
        );
        for p in &data.provinces {
            anyhow::ensure!(
                remote
                    .iter()
                    .any(|r| r.id == p.id && r.name.eq_ignore_ascii_case(&p.name)),
                "Province code/name mismatch: {}",
                p.id
            );
            let regencies: Vec<Remote> = client
                .get(format!(
                    "{}/regencies/{}.json",
                    base.trim_end_matches('/'),
                    p.id
                ))
                .send()
                .await?
                .error_for_status()?
                .json()
                .await?;
            let local: Vec<_> = data
                .cities
                .iter()
                .filter(|c| c.province_id == p.id)
                .collect();
            anyhow::ensure!(
                local.len() == regencies.len()
                    && local.iter().all(|c| regencies.iter().any(|r| r.id == c.id
                        && r.province_id == c.province_id
                        && r.name.eq_ignore_ascii_case(&c.name))),
                "City dataset differs in province {}",
                p.id
            );
        }
    }
    if a.check_only {
        println!(
            "{}",
            serde_json::json!({"status":"valid", "source":data.source, "version":data.version, "provinces":data.provinces.len(), "cities":data.cities.len(), "imported":false})
        );
        return Ok(());
    }
    let pool = sqlx::PgPool::connect(&std::env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?;
    let mut tx = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(8125602)")
        .execute(&mut *tx)
        .await?;
    for p in data.provinces {
        sqlx::query("INSERT INTO provinces(id,name,slug) VALUES($1,$2,$3) ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug").bind(p.id).bind(&p.name).bind(slug(&p.name)).execute(&mut *tx).await?;
    }
    for c in data.cities {
        sqlx::query("INSERT INTO cities(id,province_id,name,short_name,slug,latitude,longitude,tier,timezone) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT(id) DO UPDATE SET province_id=EXCLUDED.province_id,name=EXCLUDED.name,short_name=EXCLUDED.short_name,slug=EXCLUDED.slug,latitude=EXCLUDED.latitude,longitude=EXCLUDED.longitude,tier=EXCLUDED.tier,timezone=EXCLUDED.timezone,updated_at=now()")
        .bind(c.id).bind(c.province_id).bind(&c.name).bind(c.short_name).bind(slug(&c.name)).bind(c.latitude).bind(c.longitude).bind(c.tier).bind(c.timezone).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    println!(
        "Imported verified snapshot: {} ({})",
        data.source, data.version
    );
    Ok(())
}
fn slug(value: &str) -> String {
    deunicode::deunicode(value)
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
