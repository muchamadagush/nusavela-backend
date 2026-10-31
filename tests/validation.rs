use nusavela_backend::{
    ingest::classify,
    models::{https_url, validate_coordinates, DirectoryQuery},
    routes::community::Submission,
    workers::rss::{public_ip, summary},
};
use serde_json::json;
#[test]
fn coordinate_boundaries_and_zero() {
    assert!(validate_coordinates(0., 0.).is_ok());
    assert!(validate_coordinates(-90., 180.).is_ok());
    for (a, b) in [(91., 0.), (0., 181.), (f64::NAN, 0.), (0., f64::INFINITY)] {
        assert!(validate_coordinates(a, b).is_err());
    }
}
#[test]
fn filters_are_bounded() {
    assert!(DirectoryQuery::default().validate().is_ok());
    assert!(DirectoryQuery {
        lat: Some(0.),
        lng: Some(0.),
        ..Default::default()
    }
    .validate()
    .is_ok());
    for q in [
        DirectoryQuery {
            lat: Some(0.),
            ..Default::default()
        },
        DirectoryQuery {
            radius_meters: Some(f64::NAN),
            ..Default::default()
        },
        DirectoryQuery {
            page: Some(0),
            ..Default::default()
        },
        DirectoryQuery {
            sort: Some("name;DROP TABLE places".into()),
            ..Default::default()
        },
    ] {
        assert!(q.validate().is_err());
    }
}
#[test]
fn affiliate_urls_are_https_without_credentials() {
    assert!(https_url("https://example.com/ticket?a=1").is_some());
    for u in [
        "javascript:alert(1)",
        "//example.com",
        "http://example.com",
        "https://user:pass@example.com",
    ] {
        assert!(https_url(u).is_none());
    }
}
#[test]
fn summaries_strip_html_and_limit_unicode_characters() {
    assert_eq!(summary("<p>Berita <b>kota</b></p>"), "Berita kota");
    assert_eq!(summary(&"é🇮🇩".repeat(100)).chars().count(), 160);
}
#[test]
fn rss_rejects_private_and_mapped_private_addresses() {
    for ip in [
        "127.0.0.1",
        "10.1.1.1",
        "169.254.169.254",
        "100.64.0.1",
        "::1",
        "::ffff:127.0.0.1",
        "fc00::1",
        "2001:db8::1",
    ] {
        assert!(!public_ip(ip.parse().unwrap()), "{ip}");
    }
    assert!(public_ip("8.8.8.8".parse().unwrap()));
    assert!(public_ip("2606:4700:4700::1111".parse().unwrap()));
}
#[test]
fn place_classification_covers_all_directory_categories() {
    for (key, value, category) in [
        ("amenity", "hospital", "rumah-sakit"),
        ("tourism", "museum", "wisata"),
        ("amenity", "cafe", "kuliner"),
        ("railway", "station", "transportasi"),
        ("amenity", "fire_station", "layanan-publik"),
    ] {
        assert_eq!(
            classify(&[(key.into(), value.into())].into_iter().collect()),
            Some(category)
        );
    }
}
#[test]
fn submissions_require_place_and_only_message_payload() {
    let mut value = json!({"city_id":"test","place_id":null,"submission_type":"correction","submitter_name":"Tester","submitter_contact":"contact@example.com","payload":{"message":"Correct this location"}});
    assert!(serde_json::from_value::<Submission>(value.clone())
        .unwrap()
        .validate()
        .is_err());
    value["place_id"] = json!(uuid::Uuid::new_v4());
    assert!(serde_json::from_value::<Submission>(value.clone())
        .unwrap()
        .validate()
        .is_ok());
    value["payload"]["is_sponsored"] = json!(true);
    assert!(serde_json::from_value::<Submission>(value)
        .unwrap()
        .validate()
        .is_err());
}

#[tokio::test]
async fn jwt_rejects_wrong_role_issuer_expiry_signature_and_algorithm() {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
    use nusavela_backend::{
        config::Config,
        routes::admin::{authorize, Claims},
        state::AppState,
    };
    let secret = "unit-test-secret-of-at-least-32-bytes";
    let config = Config {
        database_url: "postgres://localhost/unused".into(),
        bind: "127.0.0.1:0".parse().unwrap(),
        cors: vec![],
        admin_secret: Some(secret.into()),
        admin_issuer: "issuer".into(),
        admin_audience: "audience".into(),
        weather_key: None,
        rss_enabled: false,
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy(&config.database_url)
        .unwrap();
    let state = AppState::new(pool, config).unwrap();
    let now = chrono::Utc::now().timestamp() as u64;
    for case in 0..7 {
        let mut c = Claims {
            sub: "admin-id".into(),
            role: "admin".into(),
            iss: "issuer".into(),
            aud: "audience".into(),
            exp: now + 900,
            nbf: now,
        };
        match case {
            1 => c.role = "user".into(),
            2 => c.iss = "wrong".into(),
            3 => c.exp = now - 1000,
            4 => c.aud = "wrong".into(),
            _ => (),
        }
        let header = if case == 6 {
            Header::new(Algorithm::HS384)
        } else {
            Header::default()
        };
        let key = if case == 5 {
            "wrong-signature-key"
        } else {
            secret
        };
        let token = encode(&header, &c, &EncodingKey::from_secret(key.as_bytes())).unwrap();
        let mut h = axum::http::HeaderMap::new();
        h.insert("authorization", format!("Bearer {token}").parse().unwrap());
        assert_eq!(authorize(&state, &h).is_ok(), case == 0, "JWT case {case}");
    }
}
