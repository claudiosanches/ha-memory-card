# Data and backups

Everything the app stores lives in the app's data folder, which Home
Assistant includes in its backups:

- `db/memory-card.sqlite`: the collection and everyone's play.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded covers, and photos of copies and
  hardware.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.

To keep a copy outside Home Assistant, **Export collection** under
[Import & export](import-export.md) downloads every copy as a CSV file, and
its **Hardware** tab every item of hardware. They leave out play sessions,
status history, photos and covers.
