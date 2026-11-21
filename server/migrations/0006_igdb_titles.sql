-- IGDB's name for a linked game, and the IGDB name the user chose not to use.
ALTER TABLE games ADD COLUMN igdb_title TEXT;
ALTER TABLE games ADD COLUMN igdb_title_declined TEXT;
