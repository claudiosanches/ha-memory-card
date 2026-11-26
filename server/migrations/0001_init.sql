-- Enum-like text columns (kind, category, region, format, condition, status, source, counts_as,
-- cover_source) are validated in Rust, not with CHECK constraints, so their value lists can
-- change without rebuilding tables.
--
-- Slugs name rows in library links: lowercase ASCII letters, digits and hyphens.
--
-- Game, release, copy and device ids are never reused, so an old export, a pending tag link or a
-- photo folder cannot land on a newer row.

CREATE TABLE platforms (
    id               INTEGER PRIMARY KEY,
    igdb_platform_id INTEGER UNIQUE,
    name             TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    abbreviation     TEXT,
    slug             TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    generation       INTEGER,
    family           TEXT,
    category         TEXT,
    active           INTEGER NOT NULL DEFAULT 1,
    -- Added by hand or through an import, as opposed to the ones the app ships.
    -- Only custom platforms can be deleted.
    custom           INTEGER NOT NULL DEFAULT 0,
    sort_order       INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE stores (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    slug       TEXT NOT NULL UNIQUE COLLATE NOCASE,
    notes      TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- The household's hardware. Consoles, handhelds and PCs hold copies and list
-- the platforms they play in device_platforms; controllers, accessories and
-- toys do not, and name the platform they are for in platform_id.
CREATE TABLE devices (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    name               TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    slug               TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    kind               TEXT    NOT NULL,
    model              TEXT,
    notes              TEXT,
    active             INTEGER NOT NULL DEFAULT 1,
    platform_id        INTEGER REFERENCES platforms (id) ON DELETE SET NULL,
    -- `owned`, or `wanted` for the wishlist.
    status             TEXT    NOT NULL DEFAULT 'owned',
    region             TEXT,
    condition          TEXT,
    store_id           INTEGER REFERENCES stores (id) ON DELETE SET NULL,
    -- YYYY-MM-DD.
    purchased_on       TEXT,
    -- Minor units and ISO 4217 codes, as on copies.
    price              INTEGER,
    currency           TEXT,
    estimated_value    INTEGER,
    estimated_currency TEXT,
    serial_number      TEXT,
    -- UPC or EAN printed on the box, digits only.
    barcode            TEXT,
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TABLE device_platforms (
    device_id   INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    platform_id INTEGER NOT NULL REFERENCES platforms (id) ON DELETE RESTRICT,
    PRIMARY KEY (device_id, platform_id)
) WITHOUT ROWID;

CREATE INDEX device_platforms_platform ON device_platforms (platform_id);

CREATE TABLE genres (
    id            INTEGER PRIMARY KEY,
    igdb_genre_id INTEGER UNIQUE,
    name          TEXT NOT NULL UNIQUE COLLATE NOCASE,
    slug          TEXT NOT NULL UNIQUE COLLATE NOCASE
);

CREATE TABLE game_modes (
    id                INTEGER PRIMARY KEY,
    igdb_game_mode_id INTEGER UNIQUE,
    name              TEXT NOT NULL UNIQUE COLLATE NOCASE,
    slug              TEXT NOT NULL UNIQUE COLLATE NOCASE
);

-- Home Assistant users who changed something or keep play, so the app can name them.
CREATE TABLE users (
    id           TEXT PRIMARY KEY,
    name         TEXT,
    display_name TEXT
) WITHOUT ROWID;

-- Store accounts own digital copies, apart from the device a copy is on: a
-- game bought on a PlayStation Network account and installed on a PS5 names
-- both.
CREATE TABLE accounts (
    id         INTEGER PRIMARY KEY,
    name       TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    slug       TEXT    NOT NULL UNIQUE COLLATE NOCASE,
    -- The store it buys from, such as Nintendo eShop.
    store_id   INTEGER REFERENCES stores (id) ON DELETE SET NULL,
    -- Whose account it is in the household.
    user_id    TEXT    REFERENCES users (id) ON DELETE SET NULL,
    notes      TEXT,
    active     INTEGER NOT NULL DEFAULT 1,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- The platforms whose games an account holds.
CREATE TABLE account_platforms (
    account_id  INTEGER NOT NULL REFERENCES accounts (id) ON DELETE CASCADE,
    platform_id INTEGER NOT NULL REFERENCES platforms (id) ON DELETE RESTRICT,
    PRIMARY KEY (account_id, platform_id)
) WITHOUT ROWID;

CREATE INDEX account_platforms_platform ON account_platforms (platform_id);

-- Player count columns on games and releases: NULL means unknown. A release
-- value wins over the game value; local and online are never mixed.
CREATE TABLE games (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT,
    igdb_id             INTEGER UNIQUE,
    title               TEXT    NOT NULL,
    sort_title          TEXT    NOT NULL,
    -- IGDB's name for a linked game, and the IGDB name the user chose not to use.
    igdb_title          TEXT,
    igdb_title_declined TEXT,
    -- Set when someone says the game is not on IGDB, so match reviews stop offering it.
    igdb_skipped        INTEGER NOT NULL DEFAULT 0,
    summary             TEXT,
    first_release_date  TEXT,
    developers          TEXT    NOT NULL DEFAULT '[]',
    publishers          TEXT    NOT NULL DEFAULT '[]',
    cover_path          TEXT,
    cover_source        TEXT,
    local_max_players   INTEGER,
    online_max_players  INTEGER,
    local_coop          INTEGER,
    online_coop         INTEGER,
    -- IGDB's average time to beat, in minutes: the main story, with extras,
    -- and everything.
    time_to_beat_main   INTEGER,
    time_to_beat_extras INTEGER,
    time_to_beat_full   INTEGER,
    igdb_synced_at      TEXT,
    created_by          TEXT    REFERENCES users (id) ON DELETE SET NULL,
    updated_by          TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at          TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX games_sort_title ON games (sort_title);

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

-- How IGDB relates a linked game to other IGDB games: kind is remaster,
-- remake or port when igdb_id is one of this game, or version_of when this
-- game is a version of igdb_id. Rows name IGDB ids, so a link shows up once
-- both games are in the collection, whichever was added first.
CREATE TABLE game_links (
    game_id INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    igdb_id INTEGER NOT NULL,
    kind    TEXT    NOT NULL,
    PRIMARY KEY (game_id, igdb_id, kind)
) WITHOUT ROWID;

CREATE INDEX game_links_igdb ON game_links (igdb_id);

-- Other names a game goes by, searched along with its title and release
-- titles. source is igdb for IGDB's alternative names, replaced on each
-- refresh, or user for names added by hand.
CREATE TABLE game_names (
    game_id INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    name    TEXT    NOT NULL,
    source  TEXT    NOT NULL,
    -- IGDB's note, such as "Japanese title" or "Acronym".
    comment TEXT,
    PRIMARY KEY (game_id, name)
) WITHOUT ROWID;

-- Labels people make up and put on games, to group and filter them.
CREATE TABLE labels (
    id         INTEGER PRIMARY KEY,
    name       TEXT NOT NULL UNIQUE COLLATE NOCASE,
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

CREATE TABLE releases (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id            INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    platform_id        INTEGER NOT NULL REFERENCES platforms (id) ON DELETE RESTRICT,
    region             TEXT    NOT NULL,
    title              TEXT,
    product_code       TEXT,
    release_date       TEXT,
    cover_path         TEXT,
    cover_source       TEXT,
    local_max_players  INTEGER,
    online_max_players INTEGER,
    local_coop         INTEGER,
    online_coop        INTEGER,
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

-- Product codes that differ only in case are one release, as release lookups and imports
-- treat them. Also serves lookups by (game_id), (game_id, platform_id) and
-- (game_id, platform_id, region).
CREATE UNIQUE INDEX releases_identity
    ON releases (game_id, platform_id, region, lower(COALESCE(product_code, '')));

CREATE INDEX releases_platform_region ON releases (platform_id, region);

CREATE TABLE copies (
    id                 INTEGER PRIMARY KEY AUTOINCREMENT,
    release_id         INTEGER NOT NULL REFERENCES releases (id) ON DELETE CASCADE,
    -- A copy that came inside another copy, such as one game of a collection on a
    -- single cartridge. It shares the bundle copy's status, format, device, account,
    -- store, condition, platform and region; price and purchase date stay on the bundle.
    bundle_id          INTEGER REFERENCES copies (id) ON DELETE CASCADE,
    status             TEXT    NOT NULL DEFAULT 'owned',
    device_id          INTEGER REFERENCES devices (id) ON DELETE SET NULL,
    -- The store account that owns a digital copy.
    account_id         INTEGER REFERENCES accounts (id) ON DELETE SET NULL,
    format             TEXT    NOT NULL,
    store_id           INTEGER REFERENCES stores (id) ON DELETE SET NULL,
    condition          TEXT,
    edition            TEXT,
    -- YYYY-MM-DD.
    purchased_on       TEXT,
    -- In the currency's minor unit, such as cents. Paid for owned copies,
    -- expected for wanted ones.
    price              INTEGER,
    -- ISO 4217 code, set together with price.
    currency           TEXT,
    -- What an owned copy would sell for now, in the minor unit of its own
    -- currency, which can differ from the price's.
    estimated_value    INTEGER,
    -- ISO 4217 code, set together with estimated_value.
    estimated_currency TEXT,
    -- UPC or EAN printed on the box, digits only.
    barcode            TEXT,
    notes              TEXT,
    -- Where a copy came from: `manual` when added in the app, `csv` from an
    -- import.
    source             TEXT    NOT NULL DEFAULT 'manual',
    -- When a copy became owned, so a wishlist copy marked Got it counts as added
    -- then. Kept by triggers, since every path that changes a status needs it.
    owned_at           TEXT,
    created_by         TEXT    REFERENCES users (id) ON DELETE SET NULL,
    updated_by         TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    updated_at         TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX copies_release_status_format ON copies (release_id, status, format);
CREATE INDEX copies_bundle ON copies (bundle_id);
CREATE INDEX copies_device ON copies (device_id);
CREATE INDEX copies_account ON copies (account_id);
CREATE INDEX copies_store ON copies (store_id);
CREATE INDEX copies_barcode ON copies (barcode);

CREATE TRIGGER copies_owned_on_insert AFTER INSERT ON copies
WHEN NEW.status = 'owned'
BEGIN
    UPDATE copies SET owned_at = NEW.created_at WHERE id = NEW.id;
END;

CREATE TRIGGER copies_owned_on_status AFTER UPDATE OF status ON copies
WHEN NEW.status IS NOT OLD.status
BEGIN
    UPDATE copies
       SET owned_at = CASE WHEN NEW.status = 'owned'
                           THEN strftime('%Y-%m-%dT%H:%M:%fZ', 'now') END
     WHERE id = NEW.id;
END;

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

-- Photos of a device, with paths under the covers directory as for copies.
CREATE TABLE device_photos (
    id         INTEGER PRIMARY KEY,
    device_id  INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    path       TEXT    NOT NULL,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX device_photos_device ON device_photos (device_id, id);

-- Home Assistant tags linked to copies. Scanning one logs play on its copy for
-- the person who scanned it.
CREATE TABLE nfc_tags (
    tag_id     TEXT    PRIMARY KEY,
    copy_id    INTEGER NOT NULL REFERENCES copies (id) ON DELETE CASCADE,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
) WITHOUT ROWID;

CREATE INDEX nfc_tags_copy ON nfc_tags (copy_id);

-- Play statuses. The six built-in keys are fixed so integrations can rely on
-- them; users can rename and recolor them and add their own.
CREATE TABLE play_statuses (
    key        TEXT    PRIMARY KEY,
    -- NULL keeps the built-in name, translated in the UI.
    label      TEXT,
    color      TEXT    NOT NULL,
    -- How the backlog page and suggestions treat it: backlog, playing,
    -- paused, finished or not_counted.
    counts_as  TEXT    NOT NULL,
    sort_order INTEGER NOT NULL,
    builtin    INTEGER NOT NULL DEFAULT 0
) WITHOUT ROWID;

INSERT INTO play_statuses (key, color, counts_as, sort_order, builtin) VALUES
    ('backlog', 'gray', 'backlog', 10, 1),
    ('playing', 'blue', 'playing', 20, 1),
    ('paused', 'yellow', 'paused', 30, 1),
    ('completed', 'green', 'finished', 40, 1),
    ('dropped', 'red', 'finished', 50, 1),
    ('mastered', 'grape', 'finished', 60, 1);

-- Play belongs to each person: their status for each game, their sessions and
-- their picks. The household shares the collection.

-- No row means no status. Deleting a status first moves or deletes its rows.
CREATE TABLE play_progress (
    user_id     TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id     INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    play_status TEXT    NOT NULL REFERENCES play_statuses (key),
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (user_id, game_id)
) WITHOUT ROWID;

CREATE INDEX play_progress_game ON play_progress (game_id);
CREATE INDEX play_progress_status ON play_progress (play_status);

-- Each time someone played a game. A game's last played is its latest session.
CREATE TABLE play_sessions (
    id         INTEGER PRIMARY KEY,
    user_id    TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id    INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    copy_id    INTEGER REFERENCES copies (id) ON DELETE SET NULL,
    played_on  TEXT    NOT NULL,
    minutes    INTEGER,
    notes      TEXT,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX play_sessions_person_game ON play_sessions (user_id, game_id, played_on);
CREATE INDEX play_sessions_person_day ON play_sessions (user_id, played_on);
-- Counts the sessions each person logged in the last day.
CREATE INDEX play_sessions_person_logged ON play_sessions (user_id, created_at);
CREATE INDEX play_sessions_game ON play_sessions (game_id, played_on);
CREATE INDEX play_sessions_copy ON play_sessions (copy_id);

CREATE TABLE suggestions (
    id           INTEGER PRIMARY KEY,
    user_id      TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id      INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    suggested_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX suggestions_person_game ON suggestions (user_id, game_id, suggested_at);
CREATE INDEX suggestions_time ON suggestions (suggested_at);

-- Starting data, so the app works without IGDB credentials.

INSERT INTO game_modes (igdb_game_mode_id, name, slug) VALUES
    (1, 'Single player',  'single-player'),
    (2, 'Multiplayer',    'multiplayer'),
    (3, 'Co-operative',   'co-op'),
    (4, 'Split screen',   'split-screen'),
    (5, 'MMO',            'mmo'),
    (6, 'Battle Royale',  'battle-royale');

-- Digital storefronts. Physical retailers depend on the country, so they are
-- left to the user or the CSV import.
INSERT INTO stores (name, slug) VALUES
    ('Nintendo eShop',    'nintendo-eshop'),
    ('PlayStation Store', 'playstation-store'),
    ('Microsoft Store',   'microsoft-store'),
    ('Steam',             'steam'),
    ('Epic Games Store',  'epic-games-store'),
    ('GOG',               'gog'),
    ('itch.io',           'itch-io');

-- Platforms: the common ones first, then every other IGDB platform switched off, so people
-- switch on what they own instead of typing it in. Generated from IGDB's platforms endpoint,
-- leaving out mainframes and duplicates.
INSERT INTO platforms (igdb_platform_id, name, abbreviation, slug, generation, family, category, active, sort_order) VALUES
    (6, 'PC (Microsoft Windows)', 'PC', 'pc', NULL, 'pc', 'computer', 1, 10),
    (167, 'PlayStation 5', 'PS5', 'ps5', 9, 'playstation', 'console', 1, 20),
    (48, 'PlayStation 4', 'PS4', 'ps4', 8, 'playstation', 'console', 1, 21),
    (9, 'PlayStation 3', 'PS3', 'ps3', 7, 'playstation', 'console', 0, 22),
    (8, 'PlayStation 2', 'PS2', 'ps2', 6, 'playstation', 'console', 0, 23),
    (7, 'PlayStation', 'PS1', 'ps1', 5, 'playstation', 'console', 0, 24),
    (46, 'PlayStation Vita', 'Vita', 'vita', 8, 'playstation', 'handheld', 0, 25),
    (38, 'PlayStation Portable', 'PSP', 'psp', 7, 'playstation', 'handheld', 0, 26),
    (169, 'Xbox Series X|S', 'Series X|S', 'xbox-series', 9, 'xbox', 'console', 0, 30),
    (49, 'Xbox One', 'XONE', 'xbox-one', 8, 'xbox', 'console', 0, 31),
    (12, 'Xbox 360', 'X360', 'xbox-360', 7, 'xbox', 'console', 0, 32),
    (11, 'Xbox', 'XBOX', 'xbox', 6, 'xbox', 'console', 0, 33),
    (508, 'Nintendo Switch 2', 'Switch 2', 'switch-2', 9, 'nintendo', 'console', 0, 40),
    (130, 'Nintendo Switch', 'Switch', 'switch', 8, 'nintendo', 'console', 1, 41),
    (41, 'Wii U', 'Wii U', 'wii-u', 8, 'nintendo', 'console', 1, 42),
    (5, 'Wii', 'Wii', 'wii', 7, 'nintendo', 'console', 1, 43),
    (21, 'Nintendo GameCube', 'NGC', 'gamecube', 6, 'nintendo', 'console', 0, 44),
    (4, 'Nintendo 64', 'N64', 'n64', 5, 'nintendo', 'console', 0, 45),
    (19, 'Super Nintendo Entertainment System', 'SNES', 'snes', 4, 'nintendo', 'console', 0, 46),
    (18, 'Nintendo Entertainment System', 'NES', 'nes', 3, 'nintendo', 'console', 0, 47),
    (37, 'Nintendo 3DS', '3DS', '3ds', 8, 'nintendo', 'handheld', 1, 50),
    (20, 'Nintendo DS', 'NDS', 'ds', 7, 'nintendo', 'handheld', 0, 51),
    (24, 'Game Boy Advance', 'GBA', 'gba', 6, 'nintendo', 'handheld', 0, 52),
    (22, 'Game Boy Color', 'GBC', 'gbc', 5, 'nintendo', 'handheld', 0, 53),
    (33, 'Game Boy', 'GB', 'gb', 4, 'nintendo', 'handheld', 0, 54);

INSERT INTO platforms (igdb_platform_id, name, abbreviation, slug, generation, family, category, active, sort_order) VALUES
    (471, 'Meta Quest 3', 'Meta Quest 3', 'meta-quest-3', 9, NULL, 'console', 0, 1001),
    (390, 'PlayStation VR2', 'PSVR2', 'psvr2', 9, 'playstation', 'console', 0, 1004),
    (414, 'LeapTV', NULL, 'leaptv', 8, NULL, 'console', 0, 1006),
    (72, 'Ouya', 'Ouya', 'ouya', 8, NULL, 'console', 0, 1008),
    (165, 'PlayStation VR', 'PSVR', 'psvr', 8, 'playstation', 'console', 0, 1010),
    (407, 'HyperScan', NULL, 'hyperscan', 7, NULL, 'console', 0, 1013),
    (240, 'Zeebo', NULL, 'zeebo', 7, NULL, 'console', 0, 1017),
    (507, 'Advanced Pico Beena', NULL, 'advanced-pico-beena', 6, 'sega', 'console', 0, 1018),
    (23, 'Dreamcast', 'DC', 'dc', 6, 'sega', 'console', 0, 1019),
    (478, 'Panasonic M2', NULL, 'panasonic-m2', 6, NULL, 'console', 0, 1021),
    (439, 'V.Smile', NULL, 'vsmile', 6, NULL, 'console', 0, 1023),
    (50, '3DO Interactive Multiplayer', '3DO', '3do', 5, NULL, 'console', 0, 1025),
    (416, '64DD', '64DD', '64dd', 5, 'nintendo', 'console', 0, 1026),
    (114, 'Amiga CD32', 'Amiga CD32', 'amiga-cd32', 5, 'commodore', 'console', 0, 1027),
    (476, 'Apple Pippin', NULL, 'apple-pippin', 5, NULL, 'console', 0, 1028),
    (62, 'Atari Jaguar', 'Jaguar', 'jaguar', 5, 'atari', 'console', 0, 1029),
    (410, 'Atari Jaguar CD', NULL, 'atari-jaguar-cd', 5, 'atari', 'console', 0, 1030),
    (274, 'PC-FX', NULL, 'pc-fx', 5, 'nec', 'console', 0, 1032),
    (308, 'Playdia', NULL, 'playdia', 5, NULL, 'console', 0, 1033),
    (32, 'Sega Saturn', 'Saturn', 'saturn', 5, 'sega', 'console', 0, 1035),
    (87, 'Virtual Boy', NULL, 'virtualboy', 5, 'nintendo', 'console', 0, 1036),
    (487, 'LaserActive', NULL, 'laseractive', 4, NULL, 'console', 0, 1037),
    (80, 'Neo Geo AES', NULL, 'neogeoaes', 4, 'snk', 'console', 0, 1038),
    (136, 'Neo Geo CD', NULL, 'neo-geo-cd', 4, 'snk', 'console', 0, 1039),
    (128, 'PC Engine SuperGrafx', NULL, 'supergrafx', 4, 'nec', 'console', 0, 1040),
    (306, 'Satellaview', NULL, 'satellaview', 4, 'nintendo', 'console', 0, 1041),
    (30, 'Sega 32X', 'Sega32', 'sega32', 4, 'sega', 'console', 0, 1042),
    (78, 'Sega CD', 'Sega CD', 'sega-cd', 4, 'sega', 'console', 0, 1043),
    (482, 'Sega CD 32X', NULL, 'sega-cd-32x', 4, 'sega', 'console', 0, 1044),
    (29, 'Sega Mega Drive/Genesis', 'Genesis/MegaDrive', 'genesis-slash-megadrive', 4, 'sega', 'console', 0, 1045),
    (339, 'Sega Pico', NULL, 'sega-pico', 4, 'sega', 'console', 0, 1046),
    (480, 'Super A''Can', NULL, 'super-acan', 4, NULL, 'console', 0, 1047),
    (58, 'Super Famicom', 'SFAM', 'sfam', 4, 'nintendo', 'console', 0, 1048),
    (131, 'Super NES CD-ROM System', NULL, 'super-nes-cd-rom-system', 4, 'nintendo', 'console', 0, 1049),
    (86, 'TurboGrafx-16/PC Engine', NULL, 'turbografx16--1', 4, 'nec', 'console', 0, 1051),
    (506, 'Amstrad GX4000', 'GX4000', 'gx4000', 3, NULL, 'console', 0, 1052),
    (60, 'Atari 7800', 'Atari7800', 'atari7800', 3, 'atari', 'console', 0, 1053),
    (376, 'Epoch Super Cassette Vision', NULL, 'epoch-super-cassette-vision', 3, NULL, 'console', 0, 1054),
    (99, 'Family Computer', NULL, 'famicom', 3, 'nintendo', 'console', 0, 1055),
    (51, 'Family Computer Disk System', NULL, 'fds', 3, 'nintendo', 'console', 0, 1056),
    (64, 'Sega Master System/Mark III', 'SMS', 'sms', 3, 'sega', 'console', 0, 1058),
    (84, 'SG-1000', NULL, 'sg1000', 3, 'sega', 'console', 0, 1059),
    (139, '1292 Advanced Programmable Video System', NULL, '1292-advanced-programmable-video-system', 2, NULL, 'console', 0, 1060),
    (59, 'Atari 2600', 'Atari2600', 'atari2600', 2, 'atari', 'console', 0, 1061),
    (66, 'Atari 5200', 'Atari5200', 'atari5200', 2, 'atari', 'console', 0, 1062),
    (91, 'Bally Astrocade', NULL, 'astrocade', 2, NULL, 'console', 0, 1063),
    (68, 'ColecoVision', NULL, 'colecovision', 2, NULL, 'console', 0, 1064),
    (375, 'Epoch Cassette Vision', NULL, 'epoch-cassette-vision', 2, NULL, 'console', 0, 1065),
    (127, 'Fairchild Channel F', NULL, 'fairchild-channel-f', 2, NULL, 'console', 0, 1066),
    (67, 'Intellivision', NULL, 'intellivision', 2, NULL, 'console', 0, 1067),
    (138, 'VC 4000', NULL, 'vc-4000', 2, NULL, 'console', 0, 1068),
    (70, 'Vectrex', NULL, 'vectrex', 2, NULL, 'console', 0, 1069),
    (88, 'Odyssey', NULL, 'odyssey--1', 1, NULL, 'console', 0, 1070),
    (142, 'PC-50X Family', NULL, 'pc-50x-family', 1, NULL, 'console', 0, 1071),
    (473, 'Arcadia 2001', NULL, 'arcadia-2001', NULL, NULL, 'console', 0, 1072),
    (145, 'AY-3-8603', NULL, 'ay-3-8603', NULL, NULL, 'console', 0, 1073),
    (146, 'AY-3-8605', NULL, 'ay-3-8605', NULL, NULL, 'console', 0, 1074),
    (147, 'AY-3-8606', NULL, 'ay-3-8606', NULL, NULL, 'console', 0, 1075),
    (148, 'AY-3-8607', NULL, 'ay-3-8607', NULL, NULL, 'console', 0, 1076),
    (144, 'AY-3-8710', NULL, 'ay-3-8710', NULL, NULL, 'console', 0, 1077),
    (143, 'AY-3-8760', NULL, 'ay-3-8760', NULL, NULL, 'console', 0, 1078),
    (239, 'Blu-ray Player', NULL, 'blu-ray-player', NULL, NULL, 'console', 0, 1079),
    (380, 'Casio Loopy', NULL, 'casio-loopy', NULL, NULL, 'console', 0, 1080),
    (238, 'DVD Player', NULL, 'dvd-player', NULL, NULL, 'console', 0, 1081),
    (388, 'Gear VR', 'Gear VR', 'gear-vr', NULL, NULL, 'console', 0, 1082),
    (382, 'Intellivision Amico', NULL, 'intellivision-amico', NULL, NULL, 'console', 0, 1083),
    (386, 'Meta Quest 2', 'Meta Quest 2', 'meta-quest-2', NULL, NULL, 'console', 0, 1084),
    (122, 'Nuon', NULL, 'nuon', NULL, NULL, 'console', 0, 1085),
    (387, 'Oculus Go', NULL, 'oculus-go', NULL, NULL, 'console', 0, 1086),
    (384, 'Oculus Quest', NULL, 'oculus-quest', NULL, NULL, 'console', 0, 1087),
    (385, 'Oculus Rift', NULL, 'oculus-rift', NULL, NULL, 'console', 0, 1088),
    (117, 'Philips CD-i', 'Philips CDI', 'philips-cdi', NULL, NULL, 'console', 0, 1089),
    (509, 'Polymega', NULL, 'polymega', NULL, NULL, 'console', 0, 1090),
    (479, 'Terebikko / See ''n Say Video Phone', NULL, 'terebikko-slash-see-n-say-video-phone', NULL, NULL, 'console', 0, 1091),
    (504, 'Uzebox', NULL, 'uzebox', NULL, NULL, 'console', 0, 1092),
    (381, 'Playdate', 'Playdate', 'playdate', 9, NULL, 'handheld', 0, 1093),
    (438, 'Arduboy', 'Arduboy', 'arduboy', 8, NULL, 'handheld', 0, 1094),
    (309, 'Evercade', 'Evercade', 'evercade', 8, NULL, 'handheld', 0, 1095),
    (137, 'New Nintendo 3DS', 'New 3DS', 'new-3ds', 8, 'nintendo', 'handheld', 0, 1096),
    (477, 'Panasonic Jungle', NULL, 'panasonic-jungle', 8, NULL, 'handheld', 0, 1098),
    (486, 'Digiblast', NULL, 'digiblast', 7, NULL, 'handheld', 0, 1100),
    (474, 'Gizmondo', NULL, 'gizmondo', 7, NULL, 'handheld', 0, 1101),
    (413, 'Leapster Explorer/LeadPad Explorer', NULL, 'leapster-explorer-slash-leadpad-explorer', 7, NULL, 'handheld', 0, 1102),
    (159, 'Nintendo DSi', NULL, 'nintendo-dsi', 7, 'nintendo', 'handheld', 0, 1104),
    (510, 'e-Reader / Card-e Reader', NULL, 'e-reader-slash-card-e-reader', 6, 'nintendo', 'handheld', 0, 1106),
    (412, 'Leapster', NULL, 'leapster', 6, NULL, 'handheld', 0, 1108),
    (42, 'N-Gage', 'NGage', 'ngage', 6, NULL, 'handheld', 0, 1109),
    (440, 'Visual Memory Unit / Visual Memory System', NULL, 'visual-memory-unit-slash-visual-memory-system', 6, 'sega', 'handheld', 0, 1110),
    (379, 'Game.com', NULL, 'game-dot-com', 5, NULL, 'handheld', 0, 1112),
    (119, 'Neo Geo Pocket', NULL, 'neo-geo-pocket', 5, 'snk', 'handheld', 0, 1113),
    (120, 'Neo Geo Pocket Color', NULL, 'neo-geo-pocket-color', 5, 'snk', 'handheld', 0, 1114),
    (441, 'PocketStation', NULL, 'pocketstation', 5, 'playstation', 'handheld', 0, 1115),
    (475, 'R-Zone', NULL, 'r-zone', 5, NULL, 'handheld', 0, 1116),
    (57, 'WonderSwan', 'WonderSwan', 'wonderswan', 5, NULL, 'handheld', 0, 1117),
    (123, 'WonderSwan Color', NULL, 'wonderswan-color', 5, NULL, 'handheld', 0, 1118),
    (61, 'Atari Lynx', 'Lynx', 'lynx', 4, 'atari', 'handheld', 0, 1119),
    (378, 'Gamate', 'Gamate', 'gamate', 4, NULL, 'handheld', 0, 1120),
    (408, 'Mega Duck/Cougar Boy', NULL, 'mega-duck-slash-cougar-boy', 4, NULL, 'handheld', 0, 1122),
    (35, 'Sega Game Gear', 'Game Gear', 'gamegear', 4, 'sega', 'handheld', 0, 1123),
    (415, 'Watara/QuickShot Supervision', NULL, 'watara-slash-quickshot-supervision', 4, NULL, 'handheld', 0, 1124),
    (307, 'Game & Watch', 'G&W', 'g-and-w', 2, 'nintendo', 'handheld', 0, 1125),
    (89, 'Microvision', NULL, 'microvision--1', 2, NULL, 'handheld', 0, 1126),
    (411, 'Handheld Electronic LCD', 'Handheld', 'handheld', NULL, NULL, 'handheld', 0, 1127),
    (55, 'Legacy Mobile Device', 'Mobile', 'mobile', NULL, NULL, 'handheld', 0, 1128),
    (166, 'Pokémon mini', NULL, 'pokemon-mini', NULL, 'nintendo', 'handheld', 0, 1129),
    (124, 'SwanCrystal', NULL, 'swancrystal', NULL, NULL, 'handheld', 0, 1130),
    (44, 'Tapwave Zodiac', NULL, 'zod', NULL, NULL, 'handheld', 0, 1131),
    (116, 'Acorn Archimedes', 'Acorn Archimedes', 'acorn-archimedes', NULL, NULL, 'computer', 0, 1132),
    (134, 'Acorn Electron', 'Acorn Electron', 'acorn-electron', NULL, NULL, 'computer', 0, 1133),
    (16, 'Amiga', 'Amiga', 'amiga', NULL, 'commodore', 'computer', 0, 1134),
    (25, 'Amstrad CPC', 'ACPC', 'acpc', NULL, NULL, 'computer', 0, 1135),
    (154, 'Amstrad PCW', 'APCW', 'apcw', NULL, NULL, 'computer', 0, 1136),
    (75, 'Apple II', 'Apple][', 'appleii', NULL, NULL, 'computer', 0, 1137),
    (115, 'Apple IIGS', NULL, 'apple-iigs', NULL, NULL, 'computer', 0, 1138),
    (65, 'Atari 8-bit', 'Atari8bit', 'atari8bit', NULL, 'atari', 'computer', 0, 1139),
    (63, 'Atari ST/STE', 'Atari-ST', 'atari-st', NULL, 'atari', 'computer', 0, 1140),
    (140, 'AY-3-8500', NULL, 'ay-3-8500', NULL, NULL, 'computer', 0, 1141),
    (141, 'AY-3-8610', NULL, 'ay-3-8610', NULL, NULL, 'computer', 0, 1142),
    (69, 'BBC Microcomputer System', NULL, 'bbcmicro', NULL, NULL, 'computer', 0, 1143),
    (107, 'Call-A-Computer time-shared mainframe computer system', NULL, 'call-a-computer', NULL, NULL, 'computer', 0, 1144),
    (109, 'CDC Cyber 70', NULL, 'cdccyber70', NULL, NULL, 'computer', 0, 1145),
    (93, 'Commodore 16', 'C16', 'c16', NULL, 'commodore', 'computer', 0, 1146),
    (15, 'Commodore C64/128/MAX', 'C64', 'c64', NULL, 'commodore', 'computer', 0, 1147),
    (158, 'Commodore CDTV', NULL, 'commodore-cdtv', NULL, 'commodore', 'computer', 0, 1148),
    (90, 'Commodore PET', NULL, 'cpet', NULL, 'commodore', 'computer', 0, 1149),
    (94, 'Commodore Plus/4', 'C+4', 'c-plus-4', NULL, 'commodore', 'computer', 0, 1150),
    (71, 'Commodore VIC-20', NULL, 'vic-20', NULL, 'commodore', 'computer', 0, 1151),
    (85, 'Donner Model 30', NULL, 'donner30', NULL, NULL, 'computer', 0, 1152),
    (13, 'DOS', 'DOS', 'dos', NULL, NULL, 'computer', 0, 1153),
    (153, 'Dragon 32/64', NULL, 'dragon-32-slash-64', NULL, NULL, 'computer', 0, 1154),
    (102, 'EDSAC', NULL, 'edsac--1', NULL, NULL, 'computer', 0, 1155),
    (505, 'Elektor TV Games Computer', NULL, 'elektor-tv-games-computer', NULL, NULL, 'computer', 0, 1156),
    (236, 'Exidy Sorcerer', NULL, 'exidy-sorcerer', NULL, NULL, 'computer', 0, 1157),
    (101, 'Ferranti Nimrod Computer', NULL, 'nimrod', NULL, NULL, 'computer', 0, 1158),
    (118, 'FM Towns', NULL, 'fm-towns', NULL, NULL, 'computer', 0, 1159),
    (152, 'FM-7', NULL, 'fm-7', NULL, NULL, 'computer', 0, 1160),
    (104, 'HP 2100', NULL, 'hp2100', NULL, NULL, 'computer', 0, 1161),
    (105, 'HP 3000', NULL, 'hp3000', NULL, NULL, 'computer', 0, 1162),
    (409, 'Legacy Computer', NULL, 'legacy-computer', NULL, NULL, 'computer', 0, 1163),
    (3, 'Linux', 'Linux', 'linux', NULL, 'linux', 'computer', 0, 1164),
    (14, 'Mac', 'Mac', 'mac', NULL, NULL, 'computer', 0, 1165),
    (27, 'MSX', 'MSX', 'msx', NULL, NULL, 'computer', 0, 1166),
    (53, 'MSX2', 'MSX2', 'msx2', NULL, NULL, 'computer', 0, 1167),
    (157, 'NEC PC-6000 Series', NULL, 'nec-pc-6000-series', NULL, 'nec', 'computer', 0, 1168),
    (133, 'Odyssey 2 / Videopac G7000', NULL, 'odyssey-2-slash-videopac-g7000', NULL, NULL, 'computer', 0, 1169),
    (125, 'PC-8800 Series', NULL, 'pc-8800-series', NULL, 'nec', 'computer', 0, 1171),
    (149, 'PC-9800 Series', NULL, 'pc-9800-series', NULL, 'nec', 'computer', 0, 1172),
    (95, 'PDP-1', NULL, 'pdp1', NULL, NULL, 'computer', 0, 1173),
    (96, 'PDP-10', NULL, 'pdp10', NULL, NULL, 'computer', 0, 1174),
    (108, 'PDP-11', NULL, 'pdp11', NULL, NULL, 'computer', 0, 1175),
    (97, 'PDP-8', NULL, 'pdp-8--1', NULL, NULL, 'computer', 0, 1176),
    (110, 'PLATO', NULL, 'plato--1', NULL, NULL, 'computer', 0, 1177),
    (106, 'SDS Sigma 7', NULL, 'sdssigma7', NULL, NULL, 'computer', 0, 1178),
    (374, 'Sharp MZ-2200', NULL, 'sharp-mz-2200', NULL, NULL, 'computer', 0, 1179),
    (77, 'Sharp X1', NULL, 'x1', NULL, NULL, 'computer', 0, 1180),
    (121, 'Sharp X68000', NULL, 'sharp-x68000', NULL, NULL, 'computer', 0, 1181),
    (406, 'Sinclair QL', NULL, 'sinclair-ql', NULL, NULL, 'computer', 0, 1182),
    (373, 'Sinclair ZX81', NULL, 'sinclair-zx81', NULL, NULL, 'computer', 0, 1183),
    (237, 'Sol-20', NULL, 'sol-20', NULL, NULL, 'computer', 0, 1184),
    (155, 'Tatung Einstein', NULL, 'tatung-einstein', NULL, NULL, 'computer', 0, 1185),
    (129, 'Texas Instruments TI-99', NULL, 'ti-99', NULL, NULL, 'computer', 0, 1186),
    (156, 'Thomson MO5', NULL, 'thomson-mo5', NULL, NULL, 'computer', 0, 1187),
    (481, 'Tomy Tutor / Pyuta / Grandstand Tutor', NULL, 'tomy-tutor-slash-pyuta-slash-grandstand-tutor', NULL, NULL, 'computer', 0, 1188),
    (126, 'TRS-80', NULL, 'trs-80', NULL, NULL, 'computer', 0, 1189),
    (151, 'TRS-80 Color Computer', NULL, 'trs-80-color-computer', NULL, NULL, 'computer', 0, 1190),
    (150, 'Turbografx-16/PC Engine CD', NULL, 'turbografx-16-slash-pc-engine-cd', NULL, 'nec', 'computer', 0, 1191),
    (472, 'visionOS', NULL, 'visionos', NULL, NULL, 'computer', 0, 1192),
    (26, 'ZX Spectrum', 'ZXS', 'zxs', NULL, NULL, 'computer', 0, 1193),
    (34, 'Android', 'Android', 'android', NULL, 'linux', 'mobile', 0, 1194),
    (73, 'BlackBerry OS', NULL, 'blackberry', NULL, NULL, 'mobile', 0, 1195),
    (39, 'iOS', 'iOS', 'ios', NULL, NULL, 'mobile', 0, 1196),
    (417, 'Palm OS', NULL, 'palm-os', NULL, NULL, 'mobile', 0, 1197),
    (405, 'Windows Mobile', NULL, 'windows-mobile', NULL, NULL, 'mobile', 0, 1198),
    (74, 'Windows Phone', 'Win Phone', 'winphone', NULL, NULL, 'mobile', 0, 1199),
    (52, 'Arcade', 'Arcade', 'arcade', NULL, NULL, 'arcade', 0, 1200),
    (135, 'Hyper Neo Geo 64', NULL, 'hyper-neo-geo-64', NULL, 'snk', 'arcade', 0, 1201),
    (79, 'Neo Geo MVS', NULL, 'neogeomvs', NULL, 'snk', 'arcade', 0, 1202),
    (389, 'AirConsole', NULL, 'airconsole', NULL, NULL, 'other', 0, 1203),
    (132, 'Amazon Fire TV', 'FireTV', 'firetv', NULL, NULL, 'other', 0, 1204),
    (164, 'Daydream', NULL, 'daydream', NULL, NULL, 'other', 0, 1205),
    (170, 'Google Stadia', 'Stadia', 'stadia', NULL, 'linux', 'other', 0, 1206),
    (162, 'Oculus VR', 'Oculus VR', 'oculus-vr', NULL, NULL, 'other', 0, 1207),
    (113, 'OnLive Game System', 'OnLive', 'onlive', NULL, NULL, 'other', 0, 1208),
    (372, 'OOParts', NULL, 'ooparts', NULL, NULL, 'other', 0, 1209),
    (377, 'Plug & Play', NULL, 'plug-and-play', NULL, NULL, 'other', 0, 1210),
    (163, 'SteamVR', 'Steam VR', 'steam-vr', NULL, NULL, 'other', 0, 1211),
    (47, 'Virtual Console', 'VC', 'vc', NULL, 'nintendo', 'other', 0, 1212),
    (82, 'Web browser', NULL, 'browser', NULL, NULL, 'other', 0, 1213),
    (161, 'Windows Mixed Reality', NULL, 'windows-mixed-reality', NULL, NULL, 'other', 0, 1214);

-- Fantasy consoles. IGDB files their games under PC and web browser, so they have no IGDB
-- platform to link to.
INSERT INTO platforms (igdb_platform_id, name, abbreviation, slug, generation, family, category, active, sort_order) VALUES
    (NULL, 'TIC-80', NULL, 'tic-80', NULL, NULL, 'other', 0, 1300),
    (NULL, 'PICO-8', NULL, 'pico-8', NULL, NULL, 'other', 0, 1301);
