-- Martin function source: /place_tiles/{z}/{x}/{y}. No private submission/contact data.
CREATE FUNCTION place_tiles(z integer,x integer,y integer)
RETURNS bytea LANGUAGE sql STABLE PARALLEL SAFE AS $$
  WITH bounds AS (SELECT ST_TileEnvelope(z,x,y) AS tile),
  features AS (
    SELECT p.id::text AS id,p.name,c.slug AS category_slug,p.is_sponsored,p.sponsored_tier,
      ST_AsMVTGeom(ST_Transform(p.geom::geometry,3857),bounds.tile,4096,64,true) AS geom
    FROM public_places p JOIN categories c ON p.category_id=c.id CROSS JOIN bounds
    WHERE p.is_active AND p.geom && ST_Transform(ST_Expand(bounds.tile,(ST_XMax(bounds.tile)-ST_XMin(bounds.tile))*64/4096),4326)::geography
  ) SELECT COALESCE(ST_AsMVT(features,'places',4096,'geom'),''::bytea) FROM features;
$$;
