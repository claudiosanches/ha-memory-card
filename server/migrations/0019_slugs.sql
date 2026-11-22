-- URL names for stores, devices and genres, like platforms and game modes
-- already have: lowercase ASCII letters, digits and hyphens.
ALTER TABLE stores ADD COLUMN slug TEXT NOT NULL DEFAULT '' COLLATE NOCASE;
ALTER TABLE devices ADD COLUMN slug TEXT NOT NULL DEFAULT '' COLLATE NOCASE;
ALTER TABLE genres ADD COLUMN slug TEXT NOT NULL DEFAULT '' COLLATE NOCASE;

-- Every character that is not a-z or 0-9 becomes one hyphen.
WITH RECURSIVE walk (id, rest, slug) AS (
    SELECT id, lower(name), '' FROM stores
    UNION ALL
    SELECT id, substr(rest, 2),
           CASE WHEN substr(rest, 1, 1) GLOB '[a-z0-9]' THEN slug || substr(rest, 1, 1)
                WHEN slug = '' OR slug GLOB '*-' THEN slug
                ELSE slug || '-' END
    FROM walk WHERE rest != ''
)
UPDATE stores SET slug = (SELECT trim(substr(slug, 1, 64), '-') FROM walk WHERE walk.id = stores.id AND rest = '');

WITH RECURSIVE walk (id, rest, slug) AS (
    SELECT id, lower(name), '' FROM devices
    UNION ALL
    SELECT id, substr(rest, 2),
           CASE WHEN substr(rest, 1, 1) GLOB '[a-z0-9]' THEN slug || substr(rest, 1, 1)
                WHEN slug = '' OR slug GLOB '*-' THEN slug
                ELSE slug || '-' END
    FROM walk WHERE rest != ''
)
UPDATE devices SET slug = (SELECT trim(substr(slug, 1, 64), '-') FROM walk WHERE walk.id = devices.id AND rest = '');

WITH RECURSIVE walk (id, rest, slug) AS (
    SELECT id, lower(name), '' FROM genres
    UNION ALL
    SELECT id, substr(rest, 2),
           CASE WHEN substr(rest, 1, 1) GLOB '[a-z0-9]' THEN slug || substr(rest, 1, 1)
                WHEN slug = '' OR slug GLOB '*-' THEN slug
                ELSE slug || '-' END
    FROM walk WHERE rest != ''
)
UPDATE genres SET slug = (SELECT trim(substr(slug, 1, 64), '-') FROM walk WHERE walk.id = genres.id AND rest = '');

-- Names with no usable characters, and names that fold to the same slug.
UPDATE stores SET slug = 'store-' || id
WHERE slug = '' OR EXISTS (SELECT 1 FROM stores s WHERE s.slug = stores.slug AND s.id < stores.id);
UPDATE devices SET slug = 'device-' || id
WHERE slug = '' OR EXISTS (SELECT 1 FROM devices d WHERE d.slug = devices.slug AND d.id < devices.id);
UPDATE genres SET slug = 'genre-' || id
WHERE slug = '' OR EXISTS (SELECT 1 FROM genres g WHERE g.slug = genres.slug AND g.id < genres.id);

CREATE UNIQUE INDEX stores_slug ON stores (slug);
CREATE UNIQUE INDEX devices_slug ON devices (slug);
CREATE UNIQUE INDEX genres_slug ON genres (slug);
