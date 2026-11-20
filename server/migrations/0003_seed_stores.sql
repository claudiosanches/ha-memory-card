-- Digital storefronts. Physical retailers depend on the country, so they are
-- left to the user or the CSV import. OR IGNORE keeps a store the user
-- already created under the same name.

INSERT OR IGNORE INTO stores (name) VALUES
    ('Nintendo eShop'),
    ('PlayStation Store'),
    ('Microsoft Store'),
    ('Steam'),
    ('Epic Games Store'),
    ('GOG'),
    ('itch.io');
