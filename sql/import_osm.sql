WITH batch AS (
 SELECT *,ST_SetSRID(ST_MakePoint(longitude,latitude),4326) AS point
 FROM osm_import_candidates
 WHERE (osm_type,osm_id)>($1,$2) AND (osm_type,osm_id)<=($3,$4) AND latitude IS NOT NULL
), assigned AS (
 SELECT b.*,c.id AS city_id,c.assignment
 FROM batch b
 CROSS JOIN LATERAL (
   SELECT id,assignment FROM (
     SELECT id,'boundary' AS assignment,0 AS priority,ST_Distance(geom,b.point::geography) AS distance
       FROM cities WHERE is_active AND boundary IS NOT NULL AND ST_Covers(boundary,b.point)
     UNION ALL
     SELECT id,'nearest_centroid' AS assignment,1 AS priority,ST_Distance(geom,b.point::geography) AS distance
       FROM cities WHERE is_active AND ST_DWithin(geom,b.point::geography,$5)
   ) choices ORDER BY priority,distance,id LIMIT 1
 ) c
)
INSERT INTO places(osm_type,osm_id,city_id,category_id,name,slug,address,latitude,longitude,opening_hours,is_24_hours,has_emergency,osm_snapshot,city_assignment)
SELECT a.osm_type,a.osm_id,a.city_id,cat.id,left(a.tags->>'name',255),
 concat('osm-',a.osm_type,'-',a.osm_id),
 NULLIF(concat_ws(', ',a.tags->>'addr:street',a.tags->>'addr:city'),''),a.latitude,a.longitude,
 left(a.tags->>'opening_hours',255),COALESCE(a.tags->>'opening_hours'='24/7',false),COALESCE(a.tags->>'emergency'='yes',false),a.tags,a.assignment
FROM assigned a JOIN categories cat ON cat.slug=a.category_slug
ON CONFLICT(osm_type,osm_id) DO UPDATE SET
 -- Never overwrite fields curated by admin/community/partners, or any sponsor/affiliate settings.
 osm_snapshot=EXCLUDED.osm_snapshot,
 name=CASE WHEN places.source='osm' THEN EXCLUDED.name ELSE places.name END,
 address=CASE WHEN places.source='osm' THEN EXCLUDED.address ELSE places.address END,
 latitude=CASE WHEN places.source='osm' THEN EXCLUDED.latitude ELSE places.latitude END,
 longitude=CASE WHEN places.source='osm' THEN EXCLUDED.longitude ELSE places.longitude END,
 city_id=CASE WHEN places.source='osm' THEN EXCLUDED.city_id ELSE places.city_id END,
 category_id=CASE WHEN places.source='osm' THEN EXCLUDED.category_id ELSE places.category_id END,
 opening_hours=CASE WHEN places.source='osm' THEN EXCLUDED.opening_hours ELSE places.opening_hours END,
 is_24_hours=CASE WHEN places.source='osm' THEN EXCLUDED.is_24_hours ELSE places.is_24_hours END,
 has_emergency=CASE WHEN places.source='osm' THEN EXCLUDED.has_emergency ELSE places.has_emergency END,
 city_assignment=CASE WHEN places.source='osm' THEN EXCLUDED.city_assignment ELSE places.city_assignment END,
 updated_at=now()
