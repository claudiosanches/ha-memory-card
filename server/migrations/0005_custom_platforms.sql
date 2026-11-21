-- Platforms people add, by hand or through an import, as opposed to the ones
-- the app ships. Only custom platforms can be deleted.
ALTER TABLE platforms ADD COLUMN custom INTEGER NOT NULL DEFAULT 0;

UPDATE platforms SET custom = 1
WHERE igdb_platform_id IS NULL AND slug NOT IN ('tic-80', 'pico-8');
