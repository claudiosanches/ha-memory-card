-- When a copy became owned, so a wishlist copy marked Got it counts as added
-- then. Kept by triggers, since every path that changes a status needs it.
ALTER TABLE copies ADD COLUMN owned_at TEXT;
UPDATE copies SET owned_at = created_at WHERE status = 'owned';

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
