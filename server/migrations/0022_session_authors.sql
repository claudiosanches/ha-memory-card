-- Counts the sessions each person logged in the last day.
CREATE INDEX play_sessions_created_by ON play_sessions (created_by, created_at);
