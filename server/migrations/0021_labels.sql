-- Labels people make up and put on games, to group and filter them.
CREATE TABLE labels (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    -- Names the label in library links.
    slug       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    -- A Mantine palette name, like play statuses.
    color      TEXT NOT NULL DEFAULT 'gray',
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE game_labels (
    game_id  INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    label_id INTEGER NOT NULL REFERENCES labels (id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, label_id)
) WITHOUT ROWID;

CREATE INDEX game_labels_label ON game_labels (label_id, game_id);
