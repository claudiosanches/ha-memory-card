-- Product codes that differ only in case are one release, as release lookups
-- and imports already treat them.
DROP INDEX releases_identity;
CREATE UNIQUE INDEX releases_identity
    ON releases (game_id, platform_id, region, lower(COALESCE(product_code, '')));
