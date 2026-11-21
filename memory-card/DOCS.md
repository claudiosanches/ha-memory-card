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

Releases for Japan, Korea and Europe get their regional cover when IGDB has
one. **Settings > Sync with IGDB** refreshes every linked game.

To use your own cover, click the cover of a game or a release on its page and
upload a JPEG, PNG or WebP image of up to 10 MB. IGDB never replaces an
uploaded cover. Removing it brings back IGDB's.

## Managers and viewers

Home Assistant signs you in, so Memory Card has no login of its own. By
default, everyone who opens it can change the collection.

To limit changes to some people:

1. Open Memory Card as a person who should manage it, go to **Settings** and
   copy the user ID under **Your access**.
2. In Home Assistant, open the Memory Card app, go to **Configuration**, add
   the ID to `managers` and save. Add one entry per manager.
3. Restart the app.

From then on, everyone else is a viewer. Viewers browse the library, game
pages, the backlog and duplicates, and can roll a random pick. They cannot
add, edit or delete anything, change play progress, import or export CSV
files, manage devices, stores, platforms and play statuses, or sync with
IGDB. Requests that Home Assistant does not tie to a user are refused.

To let viewers keep track of what they play, turn on
`viewers_track_progress`. They can then change play status and last played,
and start games from the backlog, but nothing else.

To let everyone change the collection again, empty `managers` and restart.

## Options

| Option | Description |
|---|---|
| `igdb_client_id` | Twitch application client ID. Empty disables IGDB. |
| `igdb_client_secret` | Twitch application client secret. |
| `log_level` | `trace`, `debug`, `info` (default), `warning` or `error`. |
| `managers` | Home Assistant user IDs that can change the collection. Empty (the default) lets everyone change it. |
| `viewers_track_progress` | Lets viewers change play status and last played. Off by default. |

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
  case. Common platforms and the digital stores (Nintendo eShop, PlayStation
  Store, Microsoft Store, Steam, Epic Games Store, GOG, itch.io) exist from
  the start. Names that match nothing are created. A new device is an account
  when all its copies are digital, otherwise a console; change it on the
  Manage screen.
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
| `play_status` | `backlog`, `playing`, `paused`, `completed`, `dropped`, `mastered`, or the name or key of a status you added. Exports write the key. |
| `platform` | Platform name, abbreviation or slug |
| `region` | `na`, `eu`, `au`, `nz`, `jp`, `cn`, `asia`, `kr`, `br`, `ww` (worldwide or region-free); also `NTSC-U`, `PAL`, `NTSC-J` |
| `release_title` | Title on this release, when it differs |
| `product_code` | Serial or product code |
| `format` | `physical` or `digital` |
| `device` | Device name |
| `store` | Store name |
| `condition` | Physical only: `sealed`, `cib`, `no_manual`, `loose` |
| `notes` | Free text |

## Play statuses

Manage > Play statuses lists the statuses in the order the app shows them.
The six built-in statuses can be renamed, recolored and reordered, but not
deleted, and their keys never change, so CSV files and integrations can rely
on them. Statuses you add have a name, a color and a "counts as" group:

- Backlog: shown in the backlog grid and picked by "Pick for me".
- Playing and On hold: shown in those sections of the Backlog page.
- Finished: counted as cleared in the backlog progress.
- Not counted: left out of the Backlog page.

Deleting a status moves its games to another status, or leaves them with none.

## Data and backups

Everything the app stores lives in the app's data folder, which Home Assistant
includes in its backups:

- `db/memory-card.sqlite`: your collection.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded cover images.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.
