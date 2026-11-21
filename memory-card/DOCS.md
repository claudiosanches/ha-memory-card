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

To keep Memory Card from contacting the internet, turn off
`external_services`. The IGDB credentials are then ignored, and games,
covers and details are entered by hand.

## Managers and viewers

Home Assistant signs you in, so Memory Card has no login of its own. Every
Home Assistant user can open it from the sidebar, administrators or not.

Only managers can change the collection. A new install has none, so it
starts read-only. To name a manager:

1. Open Memory Card as that person, go to **Settings** and copy the user ID
   under **Your access**.
2. In Home Assistant, go to **Settings > Apps > Memory Card** and open the
   **Configuration** tab. Add the ID to **Managers**, one entry per manager,
   and save. If the form shows no field for it, choose **Edit in YAML** from
   the ⋮ menu and write:

   ```yaml
   managers:
     - your-user-id
   ```

3. Restart the app from the **Info** tab.

Everyone else is a viewer. Viewers browse the library, game pages, the
backlog and duplicates. They cannot add, edit or delete anything, roll a
random pick, change play progress, import or export CSV files, manage
devices, stores, platforms and play statuses, or sync with IGDB. Requests
that Home Assistant does not tie to a user are refused.

To let viewers keep track of what they play, turn on
`viewers_track_progress`. They can then change play status, log play
sessions and start games from the backlog, but nothing else.

Memory Card keeps the name of each user who adds or changes a game or a
copy, and the game page shows who added it and who changed it last.

## Options

| Option | Description |
|---|---|
| `igdb_client_id` | Twitch application client ID. Empty disables IGDB. |
| `igdb_client_secret` | Twitch application client secret. |
| `external_services` | Lets Memory Card contact IGDB. On by default; IGDB also needs the two credentials above. |
| `log_level` | `trace`, `debug`, `info` (default), `warning` or `error`. |
| `managers` | Home Assistant user IDs that can change the collection. Empty (the default) leaves it read-only for everyone. |
| `sensors` | Publishes Home Assistant sensors through the MQTT broker. On by default; does nothing without a broker. |
| `viewers_track_progress` | Lets viewers change play status and log play sessions. Off by default. |

## Wishlist

**Wishlist** in the sidebar lists the copies you want, each with its
platform, region and format. **Add to wishlist** asks for the game first,
then the copy. A copy can also go on the wishlist from a game's page:
choose **On my wishlist** when adding it.

When you get one, **Got it** moves it into your collection and lets you
fill in the device, Bought at and condition. Wishlist copies do not count as
owned anywhere else: the library, backlog, duplicates and Missing details
only look at copies you own. **Show > Wishlist** in the library filters
lists the games on it.

## Import and export (CSV)

**Import and export** in the sidebar downloads your collection as a CSV file
and imports one back. Start from **Download blank template** to enter a
collection in a spreadsheet. **Export CSV** on the Library page downloads
only the copies that match its filters.

After an import, games without an IGDB link can be linked with
**Settings > Sync with IGDB**, which matches them by exact title.

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
| `status` | `owned` or `wanted` (empty means owned). `wishlist` is read as `wanted`. |
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

## Home Assistant sensors

Memory Card adds a **Memory Card** device to Home Assistant through MQTT
discovery. It needs the **Mosquitto broker** app and the **MQTT**
integration, which Home Assistant offers to set up once the broker runs.
Start Memory Card after the broker so it finds it; the Status section under
Settings in Memory Card says whether the sensors are connected.

| Entity | State | Attributes |
|---|---|---|
| `sensor.memory_card_games` | Games with at least one owned copy | |
| `sensor.memory_card_copies` | Owned copies | |
| `sensor.memory_card_wishlist` | Copies on the wishlist | |
| `sensor.memory_card_backlog` | Owned games in a status that counts as backlog | |
| `sensor.memory_card_playing` | Owned games in a status that counts as playing | `games`: their titles, last played first |
| `sensor.memory_card_last_played` | Title of the game with the latest play session | `date`, `platform` of the copy played |
| `sensor.memory_card_last_added` | Title of the game whose owned copy was added last | `added_at`, `platform` |

The counts keep long-term statistics, so history graphs show how the
collection grows. Sensors update a second after each change and show as
unavailable while Memory Card is stopped. Turning off the `sensors` option
removes the device on the next start.

## Data and backups

Everything the app stores lives in the app's data folder, which Home Assistant
includes in its backups:

- `db/memory-card.sqlite`: your collection.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded cover images.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.
