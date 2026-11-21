-- Home Assistant tags linked to copies. Scanning one logs a play session for
-- its copy and sets the game to playing.
CREATE TABLE nfc_tags (
    tag_id     TEXT    PRIMARY KEY,
    copy_id    INTEGER NOT NULL REFERENCES copies (id) ON DELETE CASCADE,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) WITHOUT ROWID;

CREATE INDEX nfc_tags_copy ON nfc_tags (copy_id);
