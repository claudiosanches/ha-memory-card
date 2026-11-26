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

ALTER TABLE copies ADD COLUMN account_id INTEGER REFERENCES accounts (id) ON DELETE SET NULL;

CREATE INDEX copies_account ON copies (account_id);

-- Devices that were accounts move over with their ids, platforms and copies.
INSERT INTO accounts (id, name, slug, notes, active, created_at, updated_at)
SELECT id, name, slug, notes, active, created_at, updated_at FROM devices WHERE kind = 'account';

INSERT INTO account_platforms (account_id, platform_id)
SELECT dp.device_id, dp.platform_id FROM device_platforms dp
JOIN devices d ON d.id = dp.device_id WHERE d.kind = 'account';

UPDATE copies SET account_id = device_id, device_id = NULL
WHERE device_id IN (SELECT id FROM devices WHERE kind = 'account');

DELETE FROM devices WHERE kind = 'account';
