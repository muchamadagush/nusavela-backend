-- Compare tile and feature bounds in the same planar CRS. Casting a world-size
-- polygon to geography can change its wrapped extent and discard valid points.
CREATE INDEX idx_places_tile_geom ON places
USING GIST (ST_Transform(geom::geometry,3857))
WHERE is_active AND abs(latitude) <= 85.0511287798066;

CREATE OR REPLACE FUNCTION place_tiles(z integer,x integer,y integer)
RETURNS bytea LANGUAGE sql STABLE PARALLEL SAFE AS $$
  WITH bounds AS (SELECT ST_TileEnvelope(z,x,y) AS tile),
  features AS (
    SELECT p.id::text AS id,p.name,c.slug AS category_slug,p.is_sponsored,p.sponsored_tier,
      ST_AsMVTGeom(ST_Transform(p.geom::geometry,3857),bounds.tile,4096,64,true) AS geom
    FROM public_places p JOIN categories c ON p.category_id=c.id
    JOIN cities city ON city.id=p.city_id CROSS JOIN bounds
    WHERE p.is_active AND city.is_active AND abs(p.latitude) <= 85.0511287798066
      AND ST_Transform(p.geom::geometry,3857) &&
        ST_Expand(bounds.tile,(ST_XMax(bounds.tile)-ST_XMin(bounds.tile))*64/4096)
  ) SELECT COALESCE(ST_AsMVT(features,'places',4096,'geom'),''::bytea) FROM features;
$$;
