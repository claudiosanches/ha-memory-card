-- Fantasy consoles. IGDB files their games under PC and web browser, so they
-- have no IGDB platform to link to.
INSERT OR IGNORE INTO platforms (name, slug, category, active, sort_order) VALUES
    ('TIC-80', 'tic-80', 'other', 0, 1300),
    ('PICO-8', 'pico-8', 'other', 0, 1301);
