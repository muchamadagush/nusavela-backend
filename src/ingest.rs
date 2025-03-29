use std::collections::HashMap;
pub fn classify(tags: &HashMap<String, String>) -> Option<&'static str> {
    let tag = |k: &str| tags.get(k).map(String::as_str).unwrap_or("");
    if ["hospital", "clinic", "pharmacy", "doctors", "dentist"].contains(&tag("amenity")) {
        Some("rumah-sakit")
    } else if [
        "attraction",
        "museum",
        "viewpoint",
        "theme_park",
        "zoo",
        "gallery",
    ]
    .contains(&tag("tourism"))
    {
        Some("wisata")
    } else if ["restaurant", "cafe", "fast_food", "food_court"].contains(&tag("amenity")) {
        Some("kuliner")
    } else if ["station", "halt"].contains(&tag("railway"))
        || tag("amenity") == "bus_station"
        || tag("public_transport") == "station"
        || tag("aeroway") == "aerodrome"
    {
        Some("transportasi")
    } else if ["police", "fire_station", "townhall", "post_office"].contains(&tag("amenity")) {
        Some("layanan-publik")
    } else {
        None
    }
}
