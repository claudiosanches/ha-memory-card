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
