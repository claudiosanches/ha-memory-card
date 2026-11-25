# Changelog

## 0.1.0

First release.

### Collection

- One collection for the household: games, their platform and region
  releases, and every copy, physical or digital.
- Copies keep their device, store, condition, edition, purchase date, price,
  estimated value, barcode, notes and up to 20 photos.
- Bundles: one copy holding several games, such as a collection on a single
  cartridge.
- Wishlist, with **Got it** to move a copy into the collection.
- Library as a cover grid, a compact list or a table of copies. Search
  ignores case and accents, and filters stay in the address.
- Labels to group and filter games.
- Manage devices, stores, platforms, play statuses and labels. Every IGDB
  platform is there to switch on.
- Cleanup: fill in missing details one game at a time, and list duplicates.

### IGDB

- Covers, summaries, genres, game modes, player counts and other names, with
  regional covers for Japan, Korea and Europe.
- Sync with IGDB, review close matches and review titles that differ on IGDB.
- Your own covers, which IGDB never replaces.
- Works without IGDB, and without the internet when `external_services` is
  off.

### Play

- Each person keeps their own play status, play sessions and backlog. Only
  they change it.
- **Played today**, a play history with the copy and minutes, and **Pick for
  me** from your backlog.
- Play statuses you add, each counting as backlog, playing, on hold, finished
  or not counted.
- Stats for the collection and your own play.
- Household page comparing everyone's play this month, this year or all
  time.

### Home Assistant

- Opens from the sidebar, signed in as your Home Assistant user.
- Managers change the collection and everyone else browses it. Viewers keep
  their own play with `viewers_track_progress`.
- Sensors for the household and for each person, through the Memory Card
  integration or MQTT discovery.
- Roll a pick from a button, an action or MQTT.
- NFC tags: scan a tag on a box to log play. Readers with no user, such as
  ESPHome ones, log play for the person `tag_readers` names.
- Barcodes: scan a box to see whether you own it, then add it with its name
  from UPCitemdb.
- Home Assistant backups include the collection, covers and photos. A copy of
  the database is kept before each upgrade that changes its structure.

### Import and export

- CSV import and export, with a preview first and a blank template.
- Reads files from other spreadsheets, matching common headers such as
  `Title`, `System` or `Price Paid`.
- Updates copies by `copy_id`, so an edited export imports back.

### Interface

- English, Spanish and Brazilian Portuguese.
- 12 themes, each in light and dark.
- `date_format` and `currency` options.
