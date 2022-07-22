CREATE SCHEMA AUTHORIZATION joe;

-- CREATE TABLE IF NOT EXISTS circles (
-- 	c circle primary key,
-- 	message text not null
-- 	-- EXCLUDE USING gist (c WITH &&)
-- ) INHERITS (geom.rectangles, "triangles", "foo"."bar");