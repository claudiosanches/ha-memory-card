-- A copy that came inside another copy, such as one game of a collection on a
-- single cartridge. It shares the bundle copy's status, format, device, store,
-- condition, platform and region; price and purchase date stay on the bundle.
ALTER TABLE copies ADD COLUMN bundle_id INTEGER REFERENCES copies (id) ON DELETE CASCADE;

CREATE INDEX copies_bundle ON copies (bundle_id);
