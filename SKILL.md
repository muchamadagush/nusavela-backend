---
name: nusavela-platform-builder
description: Build, style, and extend the NusaVela (nusavela.com) 514-city Indonesian local discovery platform using TanStack Start, Rust (Axum, SQLx, osmpbf), and PostgreSQL/PostGIS. Use whenever asked to create or update NusaVela pages, geospatial API endpoints, OSM data seeders, UI components, SEO metadata, or monetization features.
---
# NusaVela Platform Builder

Build and maintain **NusaVela (`nusavela.com`)**, a nationwide Indonesian local discovery and public utility platform covering 514 regencies and cities (wisata, rumah sakit/IGD 24 jam, transportasi, berita daerah, event, dan nomor darurat) powered by **TanStack Start** and **Rust (Axum + SQLx + PostGIS)**.

## When to Use

- Creating or updating TanStack Start routes under `/kota/$citySlug` or `/en/city/$citySlug`
- Writing Rust backend endpoints in Axum, PostGIS spatial queries in SQLx, or PBF data seeders with `osmpbf`
- Styling UI components using the NusaVela Archipelago Editorial Luxury design system (`DESIGN.md`)
- Implementing monetization features (NusaVela Verified/Spotlight B2B badges, Traveloka/Tiket.com affiliate CTAs, or AdSense slots)

## Core Workflow & Implementation Steps

### 1. Apply the NusaVela Design System (`DESIGN.md`)

Follow the **Archipelago Editorial Luxury + Spatial Bento Utility** aesthetic:

- **Color Tokens:**
  - Canvas Background: Warm Alabaster `#FAF8F5` (Light) / Abyssal Slate `#07111E` (Dark)
  - Card Surface: Pure Ivory `#FFFFFF` (Light) / Deep Slate `#0F1E32` (Dark) with `1px solid #E6E2D8` border
  - Primary Brand: Deep Vela Navy `#0B1F3A`
  - Secondary Brand: Emerald Nusantara `#0D5C46`
  - Monetization & Verified Accent: Swarna Gold `#C89B3C` on Champagne surface `#FDF8EC`
  - Affiliate CTA: Royal Ocean Blue `#1D4ED8`
  - Emergency / 24H ER Alert: Siaga Crimson `#DC2626` on `#FEF2F2`
- **Typography:**
  - Hero & City Titles: `Playfair Display` or `Instrument Serif` (bold editorial serif)
  - UI & Directory Cards: `Plus Jakarta Sans`
  - Distances, Prices, Coordinates & Emergency Numbers: `JetBrains Mono`
- **Responsive Layout:**
  - Desktop (`>= 1024px`): 55% directory card grid on the left, 45% sticky interactive map on the right
  - Mobile (`< 768px`): Bento grid city dashboard and draggable bottom sheet over the interactive map

### 2. Build TanStack Start Routes with Programmatic SEO & ISR

Every dynamic city or category route must include type-safe URL search parameters, SSR data loading, ISR cache headers, and dynamic SEO metadata:

1. Define a `zod` schema in `validateSearch` for map and filter state (`lat`, `lng`, `radius`, `open24h`, `page`).
2. Fetch data from the Rust API inside `loader` using `loaderDeps`.
3. Return standard HTTP ISR cache headers in `headers()`:
   `Cache-Control: public, max-age=300, s-maxage=3600, stale-while-revalidate=86400`
4. Generate dynamic `<title>`, `<meta name="description">`, Open Graph tags, and JSON-LD (`TouristAttraction`, `Hospital`, or `LocalBusiness`) inside `head({ loaderData })`.

### 3. Write Rust Geospatial Endpoints & OSM Seeders (`AGENTS.md`)

1. Use `sqlx::query_as` with PostGIS `GEOGRAPHY(Point, 4326)` columns (`geom`).
2. Always prioritize sponsored listings in directory queries before sorting by distance or rating:
   `ORDER BY p.is_sponsored DESC, p.sponsored_tier DESC, distance_meters ASC NULLS LAST`
3. When seeding 514 cities from Geofabrik `indonesia-latest.osm.pbf`, use the `osmpbf` crate with parallel iteration (`par_map_reduce`) and batch PostgreSQL upserts (`ON CONFLICT (osm_id) DO UPDATE`).
4. Truncate RSS news summaries to a maximum of 160-200 characters and always link directly to the original publisher URL.

### 4. Enforce Licensing & Public Safety Guardrails

- Always display `© OpenStreetMap contributors` on map views and OpenWeather attribution on weather widgets.
- Use self-hosted Martin vector tiles or a commercial tile provider in production rather than `tile.openstreetmap.org`.
- Never place intrusive display ads or pop-ups over the 112/119 Emergency SOS bar or 24-hour Hospital ER directory views.

## Gotchas

- Leaflet and MapLibre require browser `window` APIs. Always render map components inside client-only boundaries so TanStack Start SSR does not fail on the server.
- PostGIS `ST_MakePoint(longitude, latitude)` takes **longitude first, latitude second**. Binding `(lat, lng)` in reverse will place Indonesian coordinates in the Indian Ocean.
