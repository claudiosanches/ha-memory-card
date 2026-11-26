-- Devices are also the household's hardware. Consoles, handhelds, PCs and
-- accounts hold copies; controllers, accessories and toys do not, and name
-- the platform they are for instead.
ALTER TABLE devices ADD COLUMN platform_id INTEGER REFERENCES platforms (id) ON DELETE SET NULL;
-- `owned`, or `wanted` for the wishlist.
ALTER TABLE devices ADD COLUMN status TEXT NOT NULL DEFAULT 'owned';
ALTER TABLE devices ADD COLUMN region TEXT;
ALTER TABLE devices ADD COLUMN condition TEXT;
ALTER TABLE devices ADD COLUMN store_id INTEGER REFERENCES stores (id) ON DELETE SET NULL;
-- YYYY-MM-DD.
ALTER TABLE devices ADD COLUMN purchased_on TEXT;
-- Minor units and ISO 4217 codes, as on copies.
ALTER TABLE devices ADD COLUMN price INTEGER;
ALTER TABLE devices ADD COLUMN currency TEXT;
ALTER TABLE devices ADD COLUMN estimated_value INTEGER;
ALTER TABLE devices ADD COLUMN estimated_currency TEXT;
ALTER TABLE devices ADD COLUMN serial_number TEXT;
-- UPC or EAN printed on the box, digits only.
ALTER TABLE devices ADD COLUMN barcode TEXT;

CREATE TABLE device_photos (
    id         INTEGER PRIMARY KEY,
    device_id  INTEGER NOT NULL REFERENCES devices (id) ON DELETE CASCADE,
    path       TEXT    NOT NULL,
    created_by TEXT    REFERENCES users (id) ON DELETE SET NULL,
    created_at TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE INDEX device_photos_device ON device_photos (device_id, id);
