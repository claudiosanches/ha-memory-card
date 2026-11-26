-- IGDB's average time to beat, in minutes: the main story, with extras, and everything.
ALTER TABLE games ADD COLUMN time_to_beat_main INTEGER;
ALTER TABLE games ADD COLUMN time_to_beat_extras INTEGER;
ALTER TABLE games ADD COLUMN time_to_beat_full INTEGER;
