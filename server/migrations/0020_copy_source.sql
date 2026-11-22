-- Where a copy came from: `manual` when added in the app, `csv` from an
-- import. Store syncs add their own sources and keep the copy's id in that
-- store in source_ref, so a later sync finds the copies it made.
ALTER TABLE copies ADD COLUMN source TEXT NOT NULL DEFAULT 'manual';
ALTER TABLE copies ADD COLUMN source_ref TEXT;

CREATE UNIQUE INDEX copies_source_ref ON copies (source, source_ref) WHERE source_ref IS NOT NULL;
