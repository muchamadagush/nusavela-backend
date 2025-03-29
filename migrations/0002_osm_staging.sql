-- Rebuildable import staging; acquire advisory lock 8125603 before truncating.
CREATE UNLOGGED TABLE osm_import_nodes (
    id bigint PRIMARY KEY,
    latitude double precision NOT NULL,
    longitude double precision NOT NULL
);
CREATE UNLOGGED TABLE osm_import_candidates (
    osm_type text NOT NULL,
    osm_id bigint NOT NULL,
    tags jsonb NOT NULL,
    category_slug text NOT NULL,
    node_refs bigint[],
    latitude double precision,
    longitude double precision,
    PRIMARY KEY(osm_type,osm_id)
);
