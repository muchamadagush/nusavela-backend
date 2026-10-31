use std::process::Command;
#[tokio::test]
#[ignore = "requires isolated migrated PostGIS database; run after api_database test"]
async fn seeders_are_idempotent_and_preserve_curated_fields() {
    let url = std::env::var("TEST_DATABASE_URL").expect("Disposable TEST_DATABASE_URL required");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    // Each CLI is a separate process to exercise actual argument handling and database upserts.
    for _ in 0..2 {
        let output = tokio::task::spawn_blocking({
            let url = url.clone();
            move || {
                Command::new(env!("CARGO_BIN_EXE_seed_wilayah"))
                    .args([
                        "--file",
                        "tests/fixtures/wilayah.test.json",
                        "--expected-provinces",
                        "1",
                        "--expected-cities",
                        "2",
                    ])
                    .env("DATABASE_URL", url)
                    .output()
                    .unwrap()
            }
        })
        .await
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for cycle in 0..2 {
        let output = tokio::task::spawn_blocking({
            let url = url.clone();
            move || {
                Command::new(env!("CARGO_BIN_EXE_seed_osm"))
                    .args([
                        "--file",
                        "tests/fixtures/synthetic.osm.pbf",
                        "--batch-size",
                        "2",
                    ])
                    .env("DATABASE_URL", url)
                    .output()
                    .unwrap()
            }
        })
        .await
        .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let count: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM places WHERE osm_id=10 AND osm_type IN ('node','way')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            count, 2,
            "OSM node and way with same numeric ID must remain separate"
        );
        let emergency: bool = sqlx::query_scalar(
            "SELECT has_emergency AND is_24_hours FROM places WHERE osm_type='way' AND osm_id=10",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(emergency);
        if cycle == 0 {
            sqlx::query("UPDATE places SET name='Curated Museum',source='admin',is_sponsored=true,sponsored_tier=1,sponsored_until=now()+interval '30 days' WHERE osm_type='node' AND osm_id=10").execute(&pool).await.unwrap();
        }
    }
    let (name, sponsored): (String, bool) =
        sqlx::query_as("SELECT name,is_sponsored FROM places WHERE osm_type='node' AND osm_id=10")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(name, "Curated Museum");
    assert!(sponsored);
    let invalid = tokio::task::spawn_blocking(move || {
        Command::new(env!("CARGO_BIN_EXE_seed_wilayah"))
            .args(["--file", "tests/fixtures/wilayah.test.json"])
            .env("DATABASE_URL", url)
            .output()
            .unwrap()
    })
    .await
    .unwrap();
    assert!(
        !invalid.status.success(),
        "Synthetic 1/2 fixture must not be accepted as national 38/514 data"
    );
    pool.close().await;
}
