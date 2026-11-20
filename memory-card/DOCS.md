# Memory Card

Memory Card keeps track of the video games you own across consoles. It opens
from the Home Assistant sidebar and needs no login of its own.

## IGDB metadata and covers

The app works without IGDB: you can enter games by hand. To fetch titles,
covers and metadata, give it access to the IGDB API.

1. Enable two-factor authentication on your Twitch account (Twitch requires
   it to register applications).
2. Open the [Twitch developer console](https://dev.twitch.tv/console/apps)
   and register an application:
   - **OAuth Redirect URLs:** `http://localhost`
   - **Category:** Application Integration
   - **Client Type:** Confidential
3. Copy the **Client ID** and generate a **Client Secret**.
4. In Home Assistant, open the Memory Card app, go to **Configuration**, fill
   in both fields and save.
5. Restart the app. Options are read only at startup.

## Options

| Option | Description |
|---|---|
| `igdb_client_id` | Twitch application client ID. Empty disables IGDB. |
| `igdb_client_secret` | Twitch application client secret. |
| `log_level` | `trace`, `debug`, `info` (default), `warning` or `error`. |

## Import and export (CSV)

**Import and export** in the sidebar downloads your collection as a CSV file
and imports one back. Start from **Download blank template** to enter a
collection in a spreadsheet.

- One row per copy you own. A row with only `game_title` (and optionally
  `igdb_id` and `play_status`) adds a game with no copies.
- Column names are not case sensitive and can be in any order. Only
  `game_title` is required in every row; rows with a copy also need
  `platform`, `region` and `format`.
- Platform, device and store names match what you already have, ignoring
  case. Names that match nothing are created. A new device is an account when
  all its copies are digital, otherwise a console; change it on the Manage
  screen.
- The export fills `copy_id`. Keep it to update that copy when importing the
  file again; leave it empty to add a new copy. Importing an unchanged export
  changes nothing.
- The import shows a preview first. Rows with errors block the import until
  you fix them in the spreadsheet and choose the file again.
- Save the file as CSV UTF-8. Files that use `;` as separator work too.

| Column | Accepted values |
|---|---|
| `copy_id` | Number from the export, or empty |
| `status` | `owned` (empty means owned) |
| `igdb_id` | IGDB game id, optional |
| `game_title` | Required |
| `play_status` | `backlog`, `playing`, `paused`, `completed`, `dropped`, `mastered` |
| `platform` | Platform name, abbreviation or slug |
| `region` | `na`, `eu`, `au`, `nz`, `jp`, `cn`, `asia`, `kr`, `br`, `ww` (worldwide or region-free); also `NTSC-U`, `PAL`, `NTSC-J` |
| `release_title` | Title on this release, when it differs |
| `product_code` | Serial or product code |
| `format` | `physical` or `digital` |
| `device` | Device name |
| `store` | Store name |
| `condition` | Physical only: `sealed`, `cib`, `no_manual`, `loose` |
| `notes` | Free text |

## Data and backups

Everything the app stores lives in the app's data folder, which Home Assistant
includes in its backups:

- `db/memory-card.sqlite`: your collection.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded cover images.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.
