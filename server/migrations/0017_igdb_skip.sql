-- Set when someone says the game is not on IGDB, so match reviews stop offering it.
ALTER TABLE games ADD COLUMN igdb_skipped INTEGER NOT NULL DEFAULT 0;
