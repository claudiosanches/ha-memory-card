# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

First release.

### Added

- One collection for the household: games, their platform and region
  releases, and every copy, physical or digital.
- Copy details: device, store, condition, edition, purchase date, price,
  estimated value, barcode, notes and up to 20 photos.
- Bundles, for one copy holding several games, such as a collection on a
  single cartridge.
- Wishlist, with **Got it** to move a copy into the collection.
- Library as a cover grid, a compact list or a table of copies, with a search
  that ignores case and accents, and filters kept in the address.
- Labels to group and filter games.
- Manage pages for devices, stores, platforms, play statuses and labels, with
  every IGDB platform ready to switch on.
- Cleanup: missing details filled in one game at a time, and duplicates.
- IGDB covers, summaries, genres, game modes, player counts, other names and
  time to beat, with regional covers for Japan, Korea and Europe.
- Hours to beat as a library filter and in **Pick for me**.
- Sync with IGDB, and reviews of close matches and of titles that differ on
  IGDB.
- Uploaded covers, which IGDB never replaces.
- Use without IGDB, and without the internet when `external_services` is off.
- Personal play: each person keeps their own play status, sessions and
  backlog, and only they change it.
- **Played today**, a play history with the copy and minutes, and **Pick for
  me** from your backlog.
- Play statuses you add, each counting as backlog, playing, on hold, finished
  or not counted.
- Stats for the collection and your own play.
- Household page comparing everyone's play this month, this year or all time.
- Sidebar app, signed in as your Home Assistant user.
- Managers who change the collection while everyone else browses it, and
  `viewers_track_progress` for viewers to keep their own play.
- Sensors for the household and for each person, through the Memory Card
  integration or MQTT discovery.
- Picks rolled from a button, an action or MQTT.
- NFC tags that log play when scanned, and `tag_readers` for readers with no
  user, such as ESPHome ones.
- Barcode scans that check whether you own a box, then add it with its name
  from UPCitemdb.
- Home Assistant backups of the collection, covers and photos, and a copy of
  the database before each upgrade that changes its structure.
- CSV import and export, with a preview, a blank template, common headers
  from other spreadsheets and updates by `copy_id`.
- English, Spanish and Brazilian Portuguese.
- 12 themes, each in light and dark.
- `date_format` and `currency` options.

[Unreleased]: https://github.com/claudiosanches/ha-memory-card/commits/dev
