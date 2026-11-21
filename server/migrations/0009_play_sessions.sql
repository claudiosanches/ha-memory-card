-- Each time someone played a game. A game's last played is its latest session,
-- which replaces games.last_played_at and games.last_played_copy_id.
CREATE TABLE play_sessions (
    id         INTEGER PRIMARY KEY,
    game_id    INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    copy_id    INTEGER REFERENCES copies (id) ON DELETE SET NULL,
    played_on  TEXT    NOT NULL,
    minutes    INTEGER,
    notes      TEXT,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX play_sessions_game ON play_sessions (game_id, played_on);
CREATE INDEX play_sessions_played_on ON play_sessions (played_on);
CREATE INDEX play_sessions_copy ON play_sessions (copy_id);

INSERT INTO play_sessions (game_id, copy_id, played_on, created_at)
SELECT id, last_played_copy_id, last_played_at, updated_at
FROM games WHERE last_played_at IS NOT NULL;

UPDATE games SET last_played_at = NULL, last_played_copy_id = NULL;
DROP INDEX games_last_played_copy;
