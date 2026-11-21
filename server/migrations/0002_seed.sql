-- Starting data so the app works without IGDB credentials. The IGDB sync
-- reconciles these rows by IGDB id, then by name, and fills the ids left NULL.

INSERT INTO platforms (igdb_platform_id, name, abbreviation, slug, family, active, sort_order) VALUES
    (6,    'PC (Microsoft Windows)',              'PC',         'pc',           'pc',          1, 10),
    (167,  'PlayStation 5',                       'PS5',        'ps5',          'playstation', 1, 20),
    (48,   'PlayStation 4',                       'PS4',        'ps4',          'playstation', 1, 21),
    (9,    'PlayStation 3',                       'PS3',        'ps3',          'playstation', 0, 22),
    (8,    'PlayStation 2',                       'PS2',        'ps2',          'playstation', 0, 23),
    (7,    'PlayStation',                         'PS1',        'ps1',          'playstation', 0, 24),
    (46,   'PlayStation Vita',                    'Vita',       'vita',         'playstation', 0, 25),
    (38,   'PlayStation Portable',                'PSP',        'psp',          'playstation', 0, 26),
    (169,  'Xbox Series X|S',                     'Series X|S', 'xbox-series',  'xbox',        0, 30),
    (49,   'Xbox One',                            'XONE',       'xbox-one',     'xbox',        0, 31),
    (12,   'Xbox 360',                            'X360',       'xbox-360',     'xbox',        0, 32),
    (11,   'Xbox',                                'XBOX',       'xbox',         'xbox',        0, 33),
    (NULL, 'Nintendo Switch 2',                   'Switch 2',   'switch-2',     'nintendo',    0, 40),
    (130,  'Nintendo Switch',                     'Switch',     'switch',       'nintendo',    1, 41),
    (41,   'Wii U',                               'Wii U',      'wii-u',        'nintendo',    1, 42),
    (5,    'Wii',                                 'Wii',        'wii',          'nintendo',    1, 43),
    (21,   'Nintendo GameCube',                   'NGC',        'gamecube',     'nintendo',    0, 44),
    (4,    'Nintendo 64',                         'N64',        'n64',          'nintendo',    0, 45),
    (19,   'Super Nintendo Entertainment System', 'SNES',       'snes',         'nintendo',    0, 46),
    (18,   'Nintendo Entertainment System',       'NES',        'nes',          'nintendo',    0, 47),
    (37,   'Nintendo 3DS',                        '3DS',        '3ds',          'nintendo',    1, 50),
    (20,   'Nintendo DS',                         'NDS',        'ds',           'nintendo',    0, 51),
    (24,   'Game Boy Advance',                    'GBA',        'gba',          'nintendo',    0, 52),
    (22,   'Game Boy Color',                      'GBC',        'gbc',          'nintendo',    0, 53),
    (33,   'Game Boy',                            'GB',         'gb',           'nintendo',    0, 54);

INSERT INTO game_modes (igdb_game_mode_id, name, slug) VALUES
    (1, 'Single player',  'single-player'),
    (2, 'Multiplayer',    'multiplayer'),
    (3, 'Co-operative',   'co-op'),
    (4, 'Split screen',   'split-screen'),
    (5, 'MMO',            'mmo'),
    (6, 'Battle Royale',  'battle-royale');

-- Digital storefronts. Physical retailers depend on the country, so they are
-- left to the user or the CSV import.
INSERT INTO stores (name) VALUES
    ('Nintendo eShop'),
    ('PlayStation Store'),
    ('Microsoft Store'),
    ('Steam'),
    ('Epic Games Store'),
    ('GOG'),
    ('itch.io');
