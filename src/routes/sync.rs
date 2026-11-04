use crate::{
    error::ApiResult,
    ingest::classify,
    models::https_url,
    routes::catalog::find_city,
    state::AppState,
};
use axum::{
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Deserialize)]
struct OverpassResponse {
    #[serde(default)]
    elements: Vec<OverpassElement>,
}

#[derive(Deserialize)]
struct OverpassElement {
    #[serde(rename = "type")]
    osm_type: String,
    id: i64,
    lat: Option<f64>,
    lon: Option<f64>,
    center: Option<OverpassCenter>,
    #[serde(default)]
    tags: HashMap<String, String>,
}

#[derive(Deserialize)]
struct OverpassCenter {
    lat: f64,
    lon: f64,
}

fn make_slug(value: &str, osm_id: i64) -> String {
    let base = deunicode::deunicode(value)
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|v| !v.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    let trimmed = if base.len() > 220 { &base[..220] } else { &base };
    if trimmed.is_empty() {
        format!("poi-{osm_id}")
    } else {
        format!("{trimmed}-{osm_id}")
    }
}

async fn fetch_wikipedia_thumb_and_extract(
    client: &reqwest::Client,
    wiki_tag: Option<&str>,
    fallback_title: &str,
) -> (Option<String>, Option<String>) {
    let title = match wiki_tag {
        Some(tag) => tag.strip_prefix("id:").or_else(|| tag.strip_prefix("en:")).unwrap_or(tag),
        None => fallback_title,
    };
    let encoded: String = title
        .trim()
        .bytes()
        .map(|b| {
            if b.is_ascii_alphanumeric() || b"-_.".contains(&b) {
                (b as char).to_string()
            } else if b == b' ' {
                "_".to_owned()
            } else {
                format!("%{:02X}", b)
            }
        })
        .collect();
    let url = format!("https://id.wikipedia.org/api/rest_v1/page/summary/{encoded}");
    let Ok(resp) = client
        .get(&url)
        .header("User-Agent", "NusaVela/1.0 (https://nusavela.com; contact@nusavela.com)")
        .timeout(std::time::Duration::from_secs(4))
        .send()
        .await
    else {
        return (None, None);
    };
    if !resp.status().is_success() {
        return (None, None);
    }
    let Ok(v) = resp.json::<Value>().await else {
        return (None, None);
    };
    let thumb = v["thumbnail"]["source"]
        .as_str()
        .and_then(https_url);
    let extract = v["extract"]
        .as_str()
        .map(|s| s.chars().take(400).collect::<String>())
        .filter(|s| !s.is_empty());
    (thumb, extract)
}

pub async fn sync_city_public_api(
    State(s): State<AppState>,
    Path(slug): Path<String>,
) -> ApiResult<Value> {
    let city = find_city(&s.pool, &slug).await?;

    // Check existing count so we can report before/after
    let existing_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM places WHERE city_id = $1 AND is_active",
    )
    .bind(&city.id)
    .fetch_one(&s.pool)
    .await?;

    // Query legal OpenStreetMap Overpass API within 15km of city center
    let overpass_query = format!(
        r#"[out:json][timeout:18];
(
  nwr["tourism"~"attraction|museum|viewpoint|theme_park|zoo|gallery"](around:15000,{lat},{lng});
  nwr["historic"~"monument|memorial|castle|archaeological_site"](around:15000,{lat},{lng});
  nwr["natural"~"beach|waterfall|peak|hot_spring"](around:15000,{lat},{lng});
  nwr["amenity"~"hospital|clinic|restaurant|cafe|bus_station|ferry_terminal|townhall|police|fire_station"](around:12000,{lat},{lng});
  nwr["railway"="station"](around:15000,{lat},{lng});
  nwr["aeroway"="aerodrome"](around:25000,{lat},{lng});
);
out center tags 120;"#,
        lat = city.latitude,
        lng = city.longitude
    );

    let overpass_resp = s
        .client
        .post("https://overpass-api.de/api/interpreter")
        .header("User-Agent", "NusaVela/1.0 (https://nusavela.com)")
        .body(overpass_query)
        .timeout(std::time::Duration::from_secs(18))
        .send()
        .await;

    let elements = match overpass_resp.and_then(|r| r.error_for_status()) {
        Ok(resp) => resp
            .json::<OverpassResponse>()
            .await
            .map(|r| r.elements)
            .unwrap_or_default(),
        Err(_) => Vec::new(),
    };

    // Load category slug -> id map
    let cat_rows: Vec<(String, i32)> = sqlx::query_as("SELECT slug, id FROM categories")
        .fetch_all(&s.pool)
        .await?;
    let cat_map: HashMap<String, i32> = cat_rows.into_iter().collect();

    let mut synced = 0_usize;
    let mut wiki_lookups = 0_usize;

    for el in elements {
        let Some(name) = el
            .tags
            .get("name:id")
            .or_else(|| el.tags.get("name"))
            .map(|s| s.trim())
            .filter(|s| s.len() >= 2 && s.len() <= 250)
        else {
            continue;
        };

        let Some(cat_slug) = classify(&el.tags) else {
            continue;
        };
        let Some(&cat_id) = cat_map.get(cat_slug) else {
            continue;
        };

        let (lat, lon) = match (el.lat, el.lon, el.center) {
            (Some(a), Some(b), _) => (a, b),
            (_, _, Some(c)) => (c.lat, c.lon),
            _ => continue,
        };
        if !(-90.0..=90.0).contains(&lat) || !(-180.0..=180.0).contains(&lon) {
            continue;
        }

        let osm_type = match el.osm_type.as_str() {
            "node" | "way" | "relation" => el.osm_type.clone(),
            _ => continue,
        };

        let place_slug = make_slug(name, el.id);
        let address = el
            .tags
            .get("addr:full")
            .cloned()
            .or_else(|| {
                let street = el.tags.get("addr:street")?;
                let num = el.tags.get("addr:housenumber").map(String::as_str).unwrap_or("");
                Some(format!("{street} {num}").trim().to_owned())
            });
        let phone = el
            .tags
            .get("phone")
            .or_else(|| el.tags.get("contact:phone"))
            .map(|s| s.chars().take(50).collect::<String>());
        let opening_hours = el
            .tags
            .get("opening_hours")
            .map(|s| s.chars().take(255).collect::<String>());
        let is_24_hours = opening_hours.as_deref() == Some("24/7") || cat_slug == "rumah-sakit";
        let has_emergency = el.tags.get("emergency").map(String::as_str) == Some("yes")
            || (cat_slug == "rumah-sakit" && is_24_hours);

        // Enrich top tourism landmarks with Wikipedia photo & summary (bounded to 4 calls per sync for speed)
        let (mut image_url, mut description) = (
            el.tags.get("image").and_then(|u| https_url(u)),
            el.tags
                .get("description:id")
                .or_else(|| el.tags.get("description"))
                .cloned(),
        );
        if cat_slug == "wisata"
            && (image_url.is_none() || description.is_none())
            && wiki_lookups < 4
            && (el.tags.contains_key("wikipedia") || el.tags.contains_key("wikidata"))
        {
            wiki_lookups += 1;
            let (w_img, w_desc) = fetch_wikipedia_thumb_and_extract(
                &s.client,
                el.tags.get("wikipedia").map(String::as_str),
                name,
            )
            .await;
            if image_url.is_none() {
                image_url = w_img;
            }
            if description.is_none() {
                description = w_desc;
            }
        }

        let res = sqlx::query(
            r#"INSERT INTO places(
                osm_type, osm_id, city_id, category_id, name, slug, address,
                latitude, longitude, phone, opening_hours, is_24_hours, has_emergency,
                description, image_url, is_verified, source, city_assignment, is_active, last_verified_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,true,'osm','nearest_centroid',true,now())
            ON CONFLICT (osm_type, osm_id) DO UPDATE SET
                name = EXCLUDED.name,
                address = COALESCE(EXCLUDED.address, places.address),
                phone = COALESCE(EXCLUDED.phone, places.phone),
                opening_hours = COALESCE(EXCLUDED.opening_hours, places.opening_hours),
                description = COALESCE(EXCLUDED.description, places.description),
                image_url = COALESCE(EXCLUDED.image_url, places.image_url),
                updated_at = now()"#,
        )
        .bind(&osm_type)
        .bind(el.id)
        .bind(&city.id)
        .bind(cat_id)
        .bind(name)
        .bind(&place_slug)
        .bind(&address)
        .bind(lat)
        .bind(lon)
        .bind(&phone)
        .bind(&opening_hours)
        .bind(is_24_hours)
        .bind(has_emergency)
        .bind(&description)
        .bind(&image_url)
        .execute(&s.pool)
        .await;

        if res.is_ok() {
            synced += 1;
        }
        if synced >= 80 {
            break;
        }
    }

    let final_count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM places WHERE city_id = $1 AND is_active",
    )
    .bind(&city.id)
    .fetch_one(&s.pool)
    .await
    .unwrap_or(existing_count);

    Ok(Json(json!({
        "city_id": city.id,
        "city_slug": city.slug,
        "city_name": city.name,
        "synced_from_overpass": synced,
        "previous_places": existing_count,
        "total_active_places": final_count,
        "sources": ["OpenStreetMap Overpass API (ODbL)", "Wikipedia REST API (CC BY-SA)"]
    })))
}
