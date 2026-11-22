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

When IGDB says two linked games in your collection are versions of each
other, such as a game and its remaster, remake, port or another edition,
each game page lists the other under **Other versions**. They stay separate
games with their own copies and progress. Run a sync to fill this in for
games linked before this was added.

The library search finds a game by its title, its release titles and its
other names, ignoring case and accents, so `pokemon` finds `Pokémon`.
Linking fills in IGDB's alternative names, such as Japanese titles and
acronyms. Add your own under **Also known as** when editing a game.

To use your own cover, click the cover of a game or a release on its page and
upload a JPEG, PNG or WebP image of up to 10 MB. IGDB never replaces an
uploaded cover. Removing it brings back IGDB's.

To keep Memory Card from contacting the internet, turn off
`external_services`. The IGDB credentials are then ignored, games, covers
and details are entered by hand, and barcode scans only check your own
copies.

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

## Language

Memory Card follows your browser's language and comes in English and
Brazilian Portuguese. To pick one, go to **Settings > Appearance >
Language**; the choice is saved in that browser. The app's options in Home
Assistant are translated too.

## Options

| Option | Description |
|---|---|
| `igdb_client_id` | Twitch application client ID. Empty disables IGDB. |
| `igdb_client_secret` | Twitch application client secret. |
| `external_services` | Lets Memory Card contact IGDB and the barcode database. On by default; IGDB also needs the two credentials above. |
| `log_level` | `trace`, `debug`, `info` (default), `warning` or `error`. |
| `managers` | Home Assistant user IDs that can change the collection. Empty (the default) leaves it read-only for everyone. |
| `sensors` | Publishes Home Assistant sensors through the MQTT broker. On by default; does nothing without a broker. |
| `viewers_track_progress` | Lets viewers change play status and log play sessions. Off by default. |
| `tag_scan_finished` | What an NFC tag scan does to a completed, dropped or mastered game: `playing` (default) sets it back to playing, `keep` keeps its status and only logs the session. |

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

## Barcodes

**Scan barcode** on the Library and Wishlist pages takes a photo of the
barcode on a box. Memory Card reads it in the browser and first checks your
copies, so in a shop you can see whether you already own a game. When no
copy has the code, it asks [UPCitemdb](https://www.upcitemdb.com) for the
product name, then lets you pick the game and add the copy with the barcode,
platform and region filled in. UPCitemdb allows 100 lookups a day without an
account. With `external_services` off, only your own copies are checked. If
the photo cannot be read, type the digits under the bars instead.

## Photos of copies

To keep photos of a copy, such as the box, the cartridge or the manual,
choose **Add photos** from the copy's **⋯** menu on the game page. On a
phone this offers the camera. Each copy takes up to 20 JPEG, PNG or WebP
photos of up to 10 MB each. They are stored with the app's data, so Home
Assistant backups include them.

## Bundles

A bundle is one copy that holds several games, such as a collection on a
single cartridge. Add the bundle as a game with its copy, then use **Add
games inside this copy** on that copy. When the bundle is linked to IGDB,
Memory Card offers the games IGDB lists in it.

Each game inside gets its own copy, so it shows in the library, the backlog
and duplicates. Those copies follow the bundle copy: change the bundle's
platform, region, format, device, store, condition or wishlist status and
they change too. Price, estimated value and purchase date stay on the bundle
copy. Counts of copies, in Settings, the stats page and the Home Assistant
sensors, count a bundle once. Bundle contents are not part of the CSV export.

## Import and export (CSV)

**Import and export** in the sidebar downloads your collection as a CSV file
and imports one back. Start from **Download blank template** to enter a
collection in a spreadsheet. **Export CSV** on the Library page downloads
only the copies that match its filters.

After an import, games without an IGDB link can be linked with
**Settings > Sync with IGDB**, which matches them by exact title. **Review
close matches** next to it goes through the rest: it shows IGDB's closest
names for each game, to link with a click or mark as not on IGDB.

- One row per copy you own. A row with only `game_title` (and optionally
  `igdb_id` and `play_status`) adds a game with no copies.
- Column names are not case sensitive and can be in any order. Only
  `game_title` is required in every row; rows with a copy also need
  `platform`, `region` and `format`.
- Files from other spreadsheets work too: common headers such as `Title`,
  `Name`, `System`, `Console`, `Serial`, `UPC`, `Price Paid` or `Comments`
  are recognized. The preview shows how each header was read; pick another
  column for any it read wrong, or leave one out.
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
| `edition` | Free text, such as `Collector's Edition` |
| `device` | Device name |
| `store` | Store name |
| `condition` | Physical only: `sealed`, `cib`, `no_manual`, `loose` |
| `purchased_on` | Date bought, `YYYY-MM-DD` |
| `price` | Price paid, or expected for a wanted copy. `59.99`, `59,99` and `1.299,90` all work; the export writes `1299.90`. Needs `currency`. |
| `currency` | Three-letter ISO 4217 code, such as `USD`, `EUR` or `BRL` |
| `estimated_value` | What an owned copy would sell for now, read like `price`. Uses `estimated_currency`, or `currency` when that is empty. |
| `estimated_currency` | Three-letter code of `estimated_value`, when it differs from `currency` |
| `barcode` | UPC or EAN digits from the box |
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
| `sensor.memory_card_pick` | Title of the latest random pick, from Memory Card or Home Assistant | `suggested_at`, `platforms` of its owned copies |
| `button.memory_card_roll_pick` | Picks a random owned game from the backlog, as **Pick for me** on the Backlog page does | |

An automation can roll with filters by publishing JSON to
`memory_card/pick/roll` with the `mqtt.publish` action, for example
`{"device": [2], "min_local_players": 2}` for a game for two on device 2.
The fields match the Backlog page's picker: `play_status`, `platform`,
`device`, `genre`, `game_mode` (lists of keys or IDs) and
`min_local_players`.

The counts keep long-term statistics, so history graphs show how the
collection grows. Sensors update a second after each change and show as
unavailable while Memory Card is stopped. Turning off the `sensors` option
removes the device on the next start.

## NFC tags

Put an NFC tag on a game's box, and scanning it with the Home Assistant
companion app starts that copy: Memory Card logs a play session for today
and sets the game to playing. To link a tag, choose **Link NFC tag** from the
copy's **⋯** menu on the game page, then scan the tag within a minute. Any
tag Home Assistant reports works, including ones written under **Settings >
Tags** in Home Assistant.

Scanning the same tag again on the same day does not add another session.
A game whose status counts as finished goes back to playing too, unless the
`tag_scan_finished` option is set to `keep`.
Memory Card hears scans through the Home Assistant API, which is why the app
asks for access to it; the Status section under Settings says whether it is
listening. **Unlink NFC tags** in the same menu removes a copy's tags.

## Data and backups

Everything the app stores lives in the app's data folder, which Home Assistant
includes in its backups:

- `db/memory-card.sqlite`: your collection.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded cover images.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.
