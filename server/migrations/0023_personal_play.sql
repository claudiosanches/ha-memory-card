-- Play belongs to each person: their status for each game, their sessions and
-- their picks. The household shares the collection.
CREATE TABLE play_progress (
    user_id     TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id     INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    play_status TEXT    NOT NULL,
    updated_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    PRIMARY KEY (user_id, game_id)
) WITHOUT ROWID;

CREATE INDEX play_progress_game ON play_progress (game_id);
CREATE INDEX play_progress_status ON play_progress (play_status);

DROP INDEX games_play_status;
ALTER TABLE games DROP COLUMN play_status;

-- Sessions nobody logged have no one to belong to.
CREATE TABLE play_sessions_by_person (
    id         INTEGER PRIMARY KEY,
    user_id    TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id    INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    copy_id    INTEGER REFERENCES copies (id) ON DELETE SET NULL,
    played_on  TEXT    NOT NULL,
    minutes    INTEGER,
    notes      TEXT,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

INSERT INTO play_sessions_by_person (id, user_id, game_id, copy_id, played_on, minutes, notes, created_at)
SELECT id, created_by, game_id, copy_id, played_on, minutes, notes, created_at
FROM play_sessions WHERE created_by IS NOT NULL;

DROP TABLE play_sessions;
ALTER TABLE play_sessions_by_person RENAME TO play_sessions;

CREATE INDEX play_sessions_person_game ON play_sessions (user_id, game_id, played_on);
CREATE INDEX play_sessions_person_day ON play_sessions (user_id, played_on);
CREATE INDEX play_sessions_person_logged ON play_sessions (user_id, created_at);
CREATE INDEX play_sessions_game ON play_sessions (game_id, played_on);
CREATE INDEX play_sessions_copy ON play_sessions (copy_id);

-- Picks are each person's too; the old ones had no person.
DROP TABLE suggestions;

CREATE TABLE suggestions (
    id           INTEGER PRIMARY KEY,
    user_id      TEXT    NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    game_id      INTEGER NOT NULL REFERENCES games (id) ON DELETE CASCADE,
    suggested_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX suggestions_person_game ON suggestions (user_id, game_id, suggested_at);
CREATE INDEX suggestions_time ON suggestions (suggested_at);
