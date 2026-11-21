-- Photos of a copy, such as the box, the disc or the manual. path is
-- relative to the covers directory.
CREATE TABLE copy_photos (
    id         INTEGER PRIMARY KEY,
    copy_id    INTEGER NOT NULL REFERENCES copies (id) ON DELETE CASCADE,
    path       TEXT    NOT NULL,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX copy_photos_copy ON copy_photos (copy_id, id);
