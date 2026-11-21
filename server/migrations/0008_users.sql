-- Home Assistant users who changed something, so the app can name them.
CREATE TABLE users (
    id           TEXT PRIMARY KEY,
    name         TEXT,
    display_name TEXT
) WITHOUT ROWID;

ALTER TABLE games ADD COLUMN created_by TEXT REFERENCES users (id) ON DELETE SET NULL;
ALTER TABLE games ADD COLUMN updated_by TEXT REFERENCES users (id) ON DELETE SET NULL;
ALTER TABLE copies ADD COLUMN created_by TEXT REFERENCES users (id) ON DELETE SET NULL;
ALTER TABLE copies ADD COLUMN updated_by TEXT REFERENCES users (id) ON DELETE SET NULL;
