use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use jsonwebtoken::{encode, EncodingKey, Header};
use nusavela_backend::{config::Config, router, routes::admin::Claims, state::AppState};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

async fn call(
    app: &axum::Router,
    method: &str,
    path: &str,
    body: Value,
    token: Option<&str>,
) -> (StatusCode, Value) {
    let mut req = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json");
    if let Some(t) = token {
        req = req.header("authorization", format!("Bearer {t}"));
    }
    let r = app
        .clone()
        .oneshot(
            req.body(if method == "GET" {
                Body::empty()
            } else {
                Body::from(body.to_string())
            })
            .unwrap(),
        )
        .await
        .unwrap();
    let status = r.status();
    let bytes = r.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes))),
    )
}
#[tokio::test]
#[ignore = "requires isolated PostgreSQL/PostGIS database; run --ignored with TEST_DATABASE_URL"]
async fn api_database_contract_and_security() {
    let url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must point to a disposable empty database");
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!().run(&pool).await.unwrap();
    // Integration fixtures use a reserved prefix; never run this suite on a production database.
    sqlx::query("TRUNCATE admin_audit,monetization_clicks,community_submissions,news_feeds,events,emergency_contacts,places,cities,provinces,request_limits CASCADE").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO provinces(id,name,slug) VALUES('test','Test','test')")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("INSERT INTO cities(id,province_id,name,short_name,slug,latitude,longitude) VALUES('test','test','Kota Uji','Uji','kota-uji',0,0),('other','test','Kota Lain','Lain','kota-lain',1,1)").execute(&pool).await.unwrap();
    let mut ids = Vec::new();
    for (name, lat, sponsor, expiry, tier) in [
        ("Near", 0.001, false, -1, 0),
        ("Sponsored", 0.05, true, 1, 1),
        ("Expired", 0.02, true, -1, 2),
    ] {
        let id:Uuid=sqlx::query_scalar("INSERT INTO places(city_id,category_id,name,slug,latitude,longitude,is_sponsored,sponsored_until,sponsored_tier,is_24_hours) SELECT 'test',id,$1,lower($1),$2,0,$3,now()+make_interval(days=>$4),$5,true FROM categories WHERE slug='wisata' RETURNING id")
            .bind(name).bind(lat).bind(sponsor).bind(expiry).bind(tier as i16).fetch_one(&pool).await.unwrap();
        ids.push(id);
    }
    let secret = "integration-test-secret-at-least-32-bytes";
    let config = Config {
        database_url: url,
        bind: "127.0.0.1:0".parse().unwrap(),
        cors: vec!["http://localhost:3000".parse().unwrap()],
        admin_secret: Some(secret.into()),
        admin_issuer: "nusavela-admin".into(),
        admin_audience: "nusavela-api".into(),
        weather_key: None,
        rss_enabled: false,
    };
    let state = AppState::new(pool.clone(), config).unwrap();
    let diagnostic = nusavela_backend::routes::catalog::directory(
        axum::extract::State(state.clone()),
        axum::extract::Path(("kota-uji".to_string(), "wisata".to_string())),
        axum::extract::Query(nusavela_backend::models::DirectoryQuery {
            lat: Some(0.),
            lng: Some(0.),
            ..Default::default()
        }),
    )
    .await;
    assert!(diagnostic.is_ok(), "Directory SQL contract: {diagnostic:?}");
    let app = router(state.clone());
    let (status, cities) = call(&app, "GET", "/api/v1/cities", Value::Null, None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(cities.as_array().unwrap().len(), 2);
    assert!(cities[0]["latitude"].is_number());
    let (_, nearest) = call(
        &app,
        "GET",
        "/api/v1/cities/nearest?lat=0&lng=0",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(nearest["slug"], "kota-uji");
    sqlx::query("INSERT INTO cities(id,province_id,name,short_name,slug,latitude,longitude) VALUES('offset','test','Asymmetric','Offset','asymmetric',-6.9,107.6)").execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO places(city_id,category_id,name,slug,latitude,longitude) SELECT 'offset',id,'Asymmetric POI','asymmetric-poi',-6.9,107.6 FROM categories WHERE slug='wisata'").execute(&pool).await.unwrap();
    let (_, asymmetric) = call(
        &app,
        "GET",
        "/api/v1/cities/nearest?lat=-6.9&lng=107.6",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(
        asymmetric["id"], "offset",
        "Longitude and latitude must not be reversed"
    );
    let (status, asymmetric) = call(
        &app,
        "GET",
        "/api/v1/kota/asymmetric/wisata?lat=-6.9&lng=107.6&radius_meters=100",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(asymmetric.as_array().unwrap().len(), 1);
    for path in [
        "/api/v1/cities/nearest?lat=NaN&lng=0",
        "/api/v1/kota/kota-uji/wisata?lat=0",
        "/api/v1/kota/kota-uji/wisata?sort=invalid",
    ] {
        assert_eq!(
            call(&app, "GET", path, Value::Null, None).await.0,
            StatusCode::BAD_REQUEST
        );
    }
    let (status, items) = call(
        &app,
        "GET",
        "/api/v1/kota/kota-uji/wisata?lat=0&lng=0&radius_meters=25000&is_24_hours=false",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{items}");
    assert_eq!(items[0]["name"], "Sponsored");
    assert_eq!(items[1]["name"], "Near");
    assert_eq!(items[2]["is_sponsored"], false);
    let (_, within) = call(
        &app,
        "GET",
        "/api/v1/kota/kota-uji/wisata?lat=0&lng=0&radius_meters=1000",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(within.as_array().unwrap().len(), 1);
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/kota/kota-lain/wisata/near",
            Value::Null,
            None
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        call(&app, "GET", "/api/v1/admin/submissions", Value::Null, None)
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let now = chrono::Utc::now().timestamp() as u64;
    let token = encode(
        &Header::default(),
        &Claims {
            sub: "tester".into(),
            role: "admin".into(),
            nbf: now,
            exp: now + 900,
            iss: "nusavela-admin".into(),
            aud: "nusavela-api".into(),
        },
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap();
    let submission = json!({"city_id":"test","place_id":ids[0],"submission_type":"correction","submitter_name":"Tester","submitter_contact":"test@example.com","payload":{"message":"Update the opening hours"}});
    let (status, v) = call(
        &app,
        "POST",
        "/api/v1/submissions",
        submission.clone(),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{v}");
    let sub = v["id"].as_str().unwrap();
    let path = format!("/api/v1/admin/submissions/{sub}/review");
    let (status,v)=call(&app,"POST",&path,json!({"status":"approved","notes":"Checked with owner","patch":{"opening_hours":"08:00-17:00","is_24_hours":false}}),Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{v}");
    assert_eq!(
        call(
            &app,
            "POST",
            &path,
            json!({"status":"rejected","notes":"Repeated review"}),
            Some(&token)
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let hours: String = sqlx::query_scalar("SELECT opening_hours FROM places WHERE id=$1")
        .bind(ids[0])
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(hours, "08:00-17:00");
    let audit: i64 = sqlx::query_scalar("SELECT count(*) FROM admin_audit")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(audit, 1);
    let sponsor = format!("/api/v1/admin/places/{}/sponsorship", ids[0]);
    assert_eq!(call(&app,"PATCH",&sponsor,json!({"tier":2,"until":chrono::Utc::now()+chrono::Duration::days(30),"payment_reference":"verified-payment"}),Some(&token)).await.0,StatusCode::OK);
    assert_eq!(call(&app,"PATCH",&sponsor,json!({"tier":2,"until":chrono::Utc::now()-chrono::Duration::days(1),"payment_reference":"invalid-expiry"}),Some(&token)).await.0,StatusCode::BAD_REQUEST);
    let affiliate = format!("/api/v1/admin/places/{}/affiliate", ids[0]);
    assert_eq!(
        call(
            &app,
            "PATCH",
            &affiliate,
            json!({"hotel_url":"https://example.com/hotel","ticket_url":null}),
            Some(&token)
        )
        .await
        .0,
        StatusCode::OK
    );
    let click = json!({"city_id":"test","place_id":ids[0],"click_type":"affiliate_hotel"});
    let (status, v) = call(&app, "POST", "/api/v1/clicks", click.clone(), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["target_url"], "https://example.com/hotel");
    let mut spoofed = click;
    spoofed["target_url"] = json!("https://attacker.test");
    assert_eq!(
        call(&app, "POST", "/api/v1/clicks", spoofed, None).await.0,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/cities/kota-uji/weather",
            Value::Null,
            None
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    let (_, news) = call(
        &app,
        "GET",
        "/api/v1/cities/kota-uji/news",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(news, json!([]));
    let (_, emergency) = call(
        &app,
        "GET",
        "/api/v1/cities/kota-uji/emergency",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(emergency, json!([]));
    // Curated admin entities are validated and audited before publication.
    let (status,new_place)=call(&app,"POST","/api/v1/admin/places",json!({"city_id":"test","category_slug":"rumah-sakit","name":"Curated Clinic","slug":"curated-clinic","latitude":0.002,"longitude":0.002,"is_24_hours":true,"has_emergency":true,"is_verified":true}),Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{new_place}");
    let (_, medical) = call(
        &app,
        "GET",
        "/api/v1/kota/kota-uji/rumah-sakit?emergency_only=true",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(medical.as_array().unwrap().len(), 1);
    let (status,event)=call(&app,"POST","/api/v1/admin/events",json!({"city_id":"test","title":"Curated Event","slug":"curated-event","start_datetime":chrono::Utc::now()+chrono::Duration::hours(1),"end_datetime":chrono::Utc::now()+chrono::Duration::hours(2),"ticket_price":0}),Some(&token)).await;
    assert_eq!(status, StatusCode::OK, "{event}");
    let (_, events) = call(
        &app,
        "GET",
        "/api/v1/cities/kota-uji/events",
        Value::Null,
        None,
    )
    .await;
    assert_eq!(events.as_array().unwrap().len(), 1);
    assert_eq!(call(&app,"POST","/api/v1/admin/places",json!({"city_id":"test","category_slug":"wisata","name":"Duplicate","slug":"near","latitude":0.0,"longitude":0.0}),Some(&token)).await.0,StatusCode::CONFLICT);
    if std::env::var("TEST_SKIP_MVT").as_deref() != Ok("1") {
        let tile: Vec<u8> = sqlx::query_scalar("SELECT place_tiles(0,0,0)")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(!tile.is_empty());
        // Fixtures near Greenwich must survive world and local tile filtering.
        for (z, x, y) in [(1, 1, 0), (8, 128, 127)] {
            let bytes: Vec<u8> = sqlx::query_scalar("SELECT place_tiles($1,$2,$3)")
                .bind(z)
                .bind(x)
                .bind(y)
                .fetch_one(&pool)
                .await
                .unwrap();
            assert!(!bytes.is_empty(), "Missing tile at {z}/{x}/{y}");
        }
        let empty: Vec<u8> = sqlx::query_scalar("SELECT place_tiles(8,0,0)")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert!(empty.is_empty(), "Unpopulated tile must remain empty");
        for (name, longitude, latitude) in [
            ("Eastern edge", 179.99, 1.0),
            ("Western edge", -179.99, 1.0),
            ("Mercator north", 0.0, 85.0),
            ("Outside Mercator", 0.0, 90.0),
        ] {
            sqlx::query("INSERT INTO places(city_id,category_id,name,slug,latitude,longitude) SELECT 'test',id,$1,$1,$2,$3 FROM categories WHERE slug='wisata'")
                .bind(name).bind(latitude).bind(longitude).execute(&pool).await.unwrap();
        }
        let world: Vec<u8> = sqlx::query_scalar("SELECT place_tiles(0,0,0)")
            .fetch_one(&pool)
            .await
            .unwrap();
        for name in ["Eastern edge", "Western edge", "Mercator north"] {
            assert!(
                world.windows(name.len()).any(|s| s == name.as_bytes()),
                "Missing {name}"
            );
        }
        assert!(!world
            .windows("Outside Mercator".len())
            .any(|s| s == b"Outside Mercator"));
    } else {
        eprintln!("MVT encoding skipped explicitly: verification PostGIS build has no protobuf-c support; native CI must run without TEST_SKIP_MVT");
    }
    let contact_is_hidden: bool =
        sqlx::query_scalar("SELECT count(*)=0 FROM emergency_contacts WHERE is_active")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(contact_is_hidden);
    // Database constraints and geographic order, not mocked query results.
    let distance:f64=sqlx::query_scalar("SELECT ST_Distance(geom,ST_SetSRID(ST_MakePoint(0,0),4326)::geography) FROM places WHERE id=$1").bind(ids[0]).fetch_one(&pool).await.unwrap();
    assert!((100.0..120.0).contains(&distance));
    assert!(sqlx::query("UPDATE places SET latitude=91 WHERE id=$1")
        .bind(ids[0])
        .execute(&pool)
        .await
        .is_err());
    for _ in 0..35 {
        let _ = call(
            &app,
            "POST",
            "/api/v1/submissions",
            submission.clone(),
            None,
        )
        .await;
    }
    assert_eq!(
        call(&app, "POST", "/api/v1/submissions", submission, None)
            .await
            .0,
        StatusCode::TOO_MANY_REQUESTS
    );
    pool.close().await;
}
