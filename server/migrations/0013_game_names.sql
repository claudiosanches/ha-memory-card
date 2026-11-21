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
