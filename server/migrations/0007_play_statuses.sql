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
