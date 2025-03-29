use clap::Parser;
use nusavela_backend::ingest::classify;
use osmpbf::{Element, ElementReader};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::sync::mpsc;
#[derive(Parser)]
#[command(about = "Bounded-memory parallel OSM importer; retains curated/sponsored fields")]
struct Args {
    #[arg(long)]
    file: PathBuf,
    #[arg(long, default_value_t = 5000)]
    batch_size: usize,
    #[arg(long, default_value_t = 50000.)]
    max_city_distance: f64,
}
#[derive(Debug)]
struct Row {
    node: Option<(i64, f64, f64)>,
    candidate: Option<Value>,
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let a = Args::parse();
    anyhow::ensure!(
        (1..=10000).contains(&a.batch_size)
            && a.max_city_distance.is_finite()
            && (100.0..=200000.).contains(&a.max_city_distance),
        "Invalid batch size or city distance"
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;
    sqlx::migrate!().run(&pool).await?;
    let mut lock = pool.acquire().await?;
    let acquired: bool = sqlx::query_scalar("SELECT pg_try_advisory_lock(8125603)")
        .fetch_one(&mut *lock)
        .await?;
    anyhow::ensure!(acquired, "Another OSM import is already running");
    // Session-scoped locks must never return to the pool while still locked on early failure.
    let result = import(&pool, a).await;
    let _ = sqlx::query("SELECT pg_advisory_unlock(8125603)")
        .execute(&mut *lock)
        .await;
    result
}
async fn import(pool: &sqlx::PgPool, a: Args) -> anyhow::Result<()> {
    let active: i64 = sqlx::query_scalar("SELECT count(*) FROM cities WHERE is_active")
        .fetch_one(pool)
        .await?;
    anyhow::ensure!(active > 0, "Import the administrative snapshot before OSM");
    sqlx::query("TRUNCATE osm_import_nodes,osm_import_candidates")
        .execute(pool)
        .await?;
    let (tx, mut rx) = mpsc::channel::<Row>(a.batch_size * 2);
    let task = tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let reader = ElementReader::from_path(a.file)?;
        let failed = AtomicBool::new(false);
        reader.par_map_reduce(
            |element| {
                if failed.load(Ordering::Relaxed) {
                    return;
                }
                let row = match element {
                    Element::Node(n) => {
                        row("node", n.id(), Some((n.lat(), n.lon())), None, n.tags())
                    }
                    Element::DenseNode(n) => {
                        row("node", n.id(), Some((n.lat(), n.lon())), None, n.tags())
                    }
                    Element::Way(w) => row("way", w.id(), None, Some(w.refs().collect()), w.tags()),
                    _ => None,
                };
                if let Some(row) = row {
                    if tx.blocking_send(row).is_err() {
                        failed.store(true, Ordering::Relaxed);
                    }
                }
            },
            || (),
            |(), ()| (),
        )?;
        anyhow::ensure!(!failed.load(Ordering::Relaxed), "Import consumer stopped");
        Ok(())
    });
    let mut rows = Vec::with_capacity(a.batch_size);
    let mut total = 0_u64;
    while let Some(row) = rx.recv().await {
        rows.push(row);
        if rows.len() >= a.batch_size {
            flush(pool, &rows).await?;
            total += rows.len() as u64;
            rows.clear();
        }
    }
    task.await??;
    if !rows.is_empty() {
        flush(pool, &rows).await?;
        total += rows.len() as u64;
    }
    println!("Staged {total} OSM elements; resolving way centroids");
    // Resolve ways only after all referenced nodes have arrived; skip incomplete geometry.
    sqlx::query(r#"WITH centers AS (
      SELECT w.osm_id,avg(n.latitude) AS lat,avg(n.longitude) AS lng
      FROM osm_import_candidates w CROSS JOIN LATERAL unnest(w.node_refs) ref(id)
      JOIN osm_import_nodes n ON n.id=ref.id
      WHERE w.osm_type='way' GROUP BY w.osm_id,w.node_refs HAVING count(*)=cardinality(w.node_refs)
    ) UPDATE osm_import_candidates w SET latitude=c.lat,longitude=c.lng FROM centers c WHERE w.osm_type='way' AND w.osm_id=c.osm_id"#).execute(pool).await?;
    // Import in bounded keyset batches. The same OSM ID on node and way remains distinct.
    let mut after_type = String::new();
    let mut after_id = 0_i64;
    let mut inserted = 0_u64;
    loop {
        let batch:Vec<(String,i64)>=sqlx::query_as("SELECT osm_type,osm_id FROM osm_import_candidates WHERE (osm_type,osm_id)>($1,$2) AND latitude IS NOT NULL ORDER BY osm_type,osm_id LIMIT $3").bind(&after_type).bind(after_id).bind(a.batch_size as i64).fetch_all(pool).await?;
        let Some((last_type, last_id)) = batch.last() else {
            break;
        };
        let n = sqlx::query(include_str!("../../sql/import_osm.sql"))
            .bind(&after_type)
            .bind(after_id)
            .bind(last_type)
            .bind(last_id)
            .bind(a.max_city_distance)
            .execute(pool)
            .await?;
        inserted += n.rows_affected();
        after_type = last_type.clone();
        after_id = *last_id;
    }
    let unresolved: i64 =
        sqlx::query_scalar("SELECT count(*) FROM osm_import_candidates WHERE latitude IS NULL")
            .fetch_one(pool)
            .await?;
    println!("Upserted {inserted} POIs. {unresolved} unresolved ways skipped. Staging retained for inspection; next import rebuilds it.");
    Ok(())
}
fn row<'a>(
    kind: &str,
    id: i64,
    point: Option<(f64, f64)>,
    refs: Option<Vec<i64>>,
    tags: impl Iterator<Item = (&'a str, &'a str)>,
) -> Option<Row> {
    let tags: HashMap<String, String> = tags.map(|(k, v)| (k.to_owned(), v.to_owned())).collect();
    let node = point.map(|(lat, lng)| (id, lat, lng));
    let candidate=classify(&tags).and_then(|category|{
        let name=tags.get("name")?;
        if name.trim().is_empty(){return None;}
        Some(json!({"osm_type":kind,"osm_id":id,"category_slug":category,"tags":tags,"node_refs":refs,"latitude":point.map(|p|p.0),"longitude":point.map(|p|p.1)}))
    });
    (node.is_some() || candidate.is_some()).then_some(Row { node, candidate })
}
async fn flush(pool: &sqlx::PgPool, rows: &[Row]) -> anyhow::Result<()> {
    let nodes: Vec<_> = rows.iter().filter_map(|r| r.node).collect();
    let ids: Vec<_> = nodes.iter().map(|n| n.0).collect();
    let lats: Vec<_> = nodes.iter().map(|n| n.1).collect();
    let lngs: Vec<_> = nodes.iter().map(|n| n.2).collect();
    let candidates: Vec<_> = rows.iter().filter_map(|r| r.candidate.clone()).collect();
    let mut tx = pool.begin().await?;
    if !ids.is_empty() {
        sqlx::query("INSERT INTO osm_import_nodes(id,latitude,longitude) SELECT * FROM unnest($1::bigint[],$2::float8[],$3::float8[]) ON CONFLICT(id) DO UPDATE SET latitude=EXCLUDED.latitude,longitude=EXCLUDED.longitude").bind(ids).bind(lats).bind(lngs).execute(&mut *tx).await?;
    }
    if !candidates.is_empty() {
        sqlx::query("INSERT INTO osm_import_candidates SELECT osm_type,osm_id,tags,category_slug,node_refs,latitude,longitude FROM jsonb_to_recordset($1) AS x(osm_type text,osm_id bigint,tags jsonb,category_slug text,node_refs bigint[],latitude float8,longitude float8) ON CONFLICT(osm_type,osm_id) DO UPDATE SET tags=EXCLUDED.tags,category_slug=EXCLUDED.category_slug,node_refs=EXCLUDED.node_refs,latitude=EXCLUDED.latitude,longitude=EXCLUDED.longitude").bind(json!(candidates)).execute(&mut *tx).await?;
    }
    tx.commit().await?;
    Ok(())
}
