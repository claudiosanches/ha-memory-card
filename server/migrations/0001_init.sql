-- Enum-like text columns (kind, region, format, condition, status, play_status,
-- cover_source) are validated in Rust, not with CHECK constraints, so their value lists can
-- change without rebuilding tables.

CREATE TABLE platforms (
    id               INTEGER PRIMARY KEY,
    igdb_platform_id INTEGER UNIQUE,
    name             TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    abbreviation     TEXT,
    slug             TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    generation       INTEGER,
    family           TEXT,
    active           INTEGER NOT NULL DEFAULT 1,
    sort_order       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE devices (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    kind       TEXT    NOT NULL,
    model      TEXT,
    notes      TEXT,
    active     INTEGER NOT NULL DEFAULT 1,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE device_platforms (
    device_id   INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    platform_id INTEGER NOT NULL REFERENCES platforms (id) ON DELETE RESTRICT,
    PRIMARY KEY (device_id, platform_id)
) WITHOUT ROWID;

CREATE INDEX device_platforms_platform ON device_platforms (platform_id);

CREATE TABLE stores (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE genres (
    id            INTEGER PRIMARY KEY,
    igdb_genre_id INTEGER UNIQUE,
    name          TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE game_modes (
    id                INTEGER PRIMARY KEY,
    igdb_game_mode_id INTEGER UNIQUE,
    name              TEXT NOT NULL UNIQUE COLLATE NOCASE,
    slug              TEXT NOT NULL UNIQUE COLLATE NOCASE
);

-- Player count columns on games and releases: NULL means unknown. A release
-- value wins over the game value; local and online are never mixed.
CREATE TABLE games (
    id                  INTEGER PRIMARY KEY,
    igdb_id             INTEGER UNIQUE,
    title               TEXT    NOT NULL,
    sort_title          TEXT    NOT NULL,
    summary             TEXT,
    first_release_date  TEXT,
    developers          TEXT    NOT NULL DEFAULT '[]',
    publishers          TEXT    NOT NULL DEFAULT '[]',
    cover_path          TEXT,
    cover_source        TEXT,
    play_status         TEXT,
    last_played_at      TEXT,
    last_played_copy_id INTEGER REFERENCES copies (id) ON DELETE SET NULL,
    local_max_players   INTEGER,
    online_max_players  INTEGER,
    local_coop          INTEGER,
    online_coop         INTEGER,
    igdb_synced_at      TEXT,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX games_sort_title ON games (sort_title);
CREATE INDEX games_play_status ON games (play_status);
CREATE INDEX games_last_played_copy ON games (last_played_copy_id);

CREATE TABLE game_game_modes (
    game_id      INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    game_mode_id INTEGER NOT NULL REFERENCES game_modes (id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, game_mode_id)
) WITHOUT ROWID;

CREATE INDEX game_game_modes_mode ON game_game_modes (game_mode_id, game_id);

CREATE TABLE game_genres (
    game_id  INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    genre_id INTEGER NOT NULL REFERENCES genres (id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, genre_id)
) WITHOUT ROWID;

CREATE INDEX game_genres_genre ON game_genres (genre_id, game_id);

CREATE TABLE releases (
    id           INTEGER PRIMARY KEY,
    game_id      INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    platform_id  INTEGER NOT NULL REFERENCES platforms (id) ON DELETE RESTRICT,
    region       TEXT    NOT NULL,
    title        TEXT,
    product_code TEXT,
    release_date TEXT,
    cover_path         TEXT,
    cover_source       TEXT,
    local_max_players  INTEGER,
    online_max_players INTEGER,
    local_coop         INTEGER,
    online_coop        INTEGER,
    created_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Also serves lookups by (game_id), (game_id, platform_id) and (game_id, platform_id, region).
CREATE UNIQUE INDEX releases_identity
    ON releases (game_id, platform_id, region, COALESCE(product_code, ''));

CREATE INDEX releases_platform_region ON releases (platform_id, region);

CREATE TABLE copies (
    id         INTEGER PRIMARY KEY,
    release_id INTEGER NOT NULL REFERENCES releases (id) ON DELETE CASCADE,
    status     TEXT    NOT NULL DEFAULT 'owned',
    device_id  INTEGER REFERENCES devices (id) ON DELETE SET NULL,
    format     TEXT    NOT NULL,
    store_id   INTEGER REFERENCES stores (id) ON DELETE SET NULL,
    condition  TEXT,
    notes      TEXT,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX copies_release_status_format ON copies (release_id, status, format);
CREATE INDEX copies_device ON copies (device_id);
CREATE INDEX copies_store ON copies (store_id);

CREATE TABLE suggestions (
    id           INTEGER PRIMARY KEY,
    game_id      INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    suggested_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX suggestions_game ON suggestions (game_id, suggested_at);
CREATE INDEX suggestions_time ON suggestions (suggested_at);

CREATE TABLE settings (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) WITHOUT ROWID;
