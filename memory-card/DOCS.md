# Memory Card

Memory Card keeps track of the video games you own across consoles, physical
or digital. It opens from the Home Assistant sidebar.

This page covers the setup and the options. The
[full guide](https://ha-memory-card.pages.dev/docs/)
goes through every page step by step, with screenshots and the themes.

![The library, with covers from IGDB](https://ha-memory-card.pages.dev/images/library.webp)

_Game titles and cover art belong to their respective owners._

## Installation

[![Open your Home Assistant instance and show the add app repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card)

1. Select the button above to add the app's repository to Home Assistant. Or
   go to **Settings > Apps > Install app**, open the ⋮ menu, choose
   **Repositories** and add `https://github.com/claudiosanches/ha-memory-card`.
2. Find **Memory Card** in the list, select it and choose **Install**.
3. Start the app. It shows in the sidebar.

A new install is read-only until you name a manager, so do that first. IGDB
credentials are optional; without them you enter games by hand.

## Your household

Memory Card keeps one collection for the whole household, and each person's
play apart.

- The collection is shared: games, copies, hardware, stores, platforms,
  labels, the list of play statuses, NFC tags, photos and covers. Everyone can
  browse it, and managers change it.
- Play is each person's own: their play status for every game, which makes
  their backlog, their play sessions and their picks from **Pick for me**.
  Only they change it, managers included. Deleting a game removes everyone's
  play for it.
- **Stats** sum up the collection and your own play, and **Household**
  compares everyone's.

A person is a Home Assistant user. Home Assistant signs you in, so Memory
Card has no login of its own.

## Managers and viewers

Every Home Assistant user can open Memory Card from the sidebar,
administrators or not.

Only managers can change the collection. A new install has none, so it
starts read-only. To name a manager:

1. Open Memory Card as that person, go to **Settings** and copy the user ID
   under **Status > Your access**.
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
backlog, duplicates, stats and the household. They cannot change the
collection or its hardware, import or export CSV files, manage accounts,
stores, platforms, play statuses and labels, or sync with IGDB. Requests that Home Assistant
does not tie to a user are refused.

Managers keep their own play. To let viewers keep theirs, turn on
`viewers_track_progress`: they can then set their play status, log their
play sessions and roll picks from their own backlog, but nothing else.

Memory Card keeps the name of each user who adds or changes a game or a
copy, and the game page shows who added it and who changed it last.

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
3. Open the application with **Manage**, copy the **Client ID** and click
   **New Secret** to generate a **Client Secret**.
4. In Home Assistant, open the Memory Card app, go to **Configuration**, fill
   in both fields and save. To keep them in Home Assistant's `secrets.yaml`
   instead, enter `!secret` and the key, such as `!secret igdb_client_secret`.
5. Restart the app. Options are read only at startup.

Releases for Japan, Korea and Europe get their regional cover when IGDB has
one. IGDB also gives the time to beat a game: the main story, with extras and
everything, shown on the game page and the backlog. **Cleanup > IGDB > Sync
with IGDB** refreshes every linked game.

To keep Memory Card from contacting the internet, turn off
`external_services`. The IGDB credentials are then ignored, games, covers
and details are entered by hand, and barcode scans only check your own
copies.

## Options

Memory Card reads its options when it starts, so restart it after changing
one.

| Option | Description |
|---|---|
| `managers` | Home Assistant user IDs that can change the collection. Empty (the default) leaves it read-only for everyone. |
| `viewers_track_progress` | Lets viewers keep their own play: play status, sessions and picks. Off by default. |
| `external_services` | Lets Memory Card contact IGDB and the barcode database. On by default; IGDB also needs the two credentials below. |
| `igdb_client_id` | Twitch application client ID. Empty disables IGDB. |
| `igdb_client_secret` | Twitch application client secret. |
| `sensors` | `off` (the default), `integration` for the Memory Card integration from the Marketplace, or `mqtt` for the MQTT broker (does nothing without one). See [Home Assistant sensors](#home-assistant-sensors). Going back to `off` removes them. |
| `tag_scan_finished` | What an NFC tag scan does when your status for its game is completed, dropped or mastered: `playing` (default) sets it back to playing, `keep` keeps it and only logs the session. |
| `tag_readers` | NFC readers with no Home Assistant user, such as ESPHome ones, and whose play their scans log. See [NFC tags](#nfc-tags). |
| `currency` | Three-letter code, such as `EUR`, that new prices start with. Empty (the default) uses the copy priced last, then Home Assistant's currency. |
| `date_format` | How dates are written: `language` (default) as the app's language writes them, `dmy` as 23/11/2026, `mdy` as 11/23/2026 or `ymd` as 2026-11-23. |
| `log_level` | `trace`, `debug`, `info` (default), `warning` or `error`. |

## Library and games

![A game page with its progress, details and labels](https://ha-memory-card.pages.dev/images/game.webp)

_Game titles and cover art belong to their respective owners._

The library shows your games as a grid of covers, a list or a table. The
filters narrow it by platform, your play status, label, genre, game mode,
device, account, store, region, format, players on the same screen and hours
to beat the main story, and they stay in the address, so a filtered library
can be bookmarked. **Show** picks between the games you own, the wishlist and
both.

The library search finds a game by its title, its release titles and its
other names, ignoring case and accents, so `pokemon` finds `Pokémon`.
Linking fills in IGDB's alternative names, such as Japanese titles and
acronyms. Add your own under **Also known as** when editing a game.

When IGDB says two linked games in your collection are versions of each
other, such as a game and its remaster, remake, port or another edition,
each game page lists the other under **Other versions**. They stay separate
games with their own copies and progress.

To use your own cover, click the cover of a game or a release on its page and
upload a JPEG, PNG or WebP image of up to 10 MB and 100 megapixels. IGDB
never replaces an uploaded cover. Removing it brings back IGDB's.

## Backlog and play statuses

![The backlog with a random pick](https://ha-memory-card.pages.dev/images/backlog.webp)

_Game titles and cover art belong to their respective owners._

**Backlog** in the sidebar shows what you are playing, what is on hold and
the games waiting for you, with how many you have cleared. It is yours:
everyone in the house has their own. **Pick for me** rolls a random game
from your backlog, for one device or a short game if you choose. A game
picked for you in the last day comes up again only when nothing else is
left. Games you have given no play status wait under **Not sorted yet**
until you give them one. Logging a session for a game in your backlog, or
with no status, sets it to playing.

Manage > Play statuses lists the statuses in the order the app shows them.
The six built-in statuses can be renamed, recolored and reordered, but not
deleted, and their keys never change, so CSV files and integrations can rely
on them. Statuses you add have a name, a color and a "counts as" group:

- Backlog: shown in the backlog grid and picked by "Pick for me".
- Playing and On hold: shown in those sections of the Backlog page.
- Finished: counted as cleared in the backlog progress.
- Not counted: left out of the Backlog page.

Deleting a status moves everyone's games in it to another status, or leaves
them with none.

## Labels

Labels are your own groups for games, such as "Couch co-op" or "Replay
someday". Manage > Labels lists them with how many games have each, and
each has a name and a color. Set a game's labels in its edit form; the game
page shows them after the summary, and selecting one opens the library
filtered by it. The library's Label filter shows games with any of the
labels you pick. Names cannot contain commas or semicolons, which separate
labels in CSV files. Deleting a label keeps its games.

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

## Hardware

**Hardware** in the sidebar lists the consoles, controllers, accessories and
toys, such as amiibo, the household owns or wants. Consoles, handhelds and
PCs hold games: each copy can say which one it lives on, and **Plays** lists
the platforms they run. Store accounts, such as Steam, are under **Manage >
Accounts**: a digital copy names the account that owns it and, apart from
that, the device it is installed on. Each account can name its store, whose
account it is and the platforms it holds. Controllers, accessories and
toys hold no games and name the platform they are for. Each item keeps its
region, condition, where and when it was bought, what it cost and what it
would sell for now, serial number, barcode, notes and up to 20 photos.
Hardware on the wishlist also shows on the Wishlist page, and **Stats** sums
up the hardware apart from the games.

## Barcodes

**Scan barcode** on the Library and Wishlist pages reads the barcode on a
box. When you open Home Assistant over HTTPS, it reads the code live from the
camera. Over plain HTTP, browsers keep the camera from web pages, so it takes
a photo instead. Memory Card reads the code in the browser and first checks
your copies, so in a shop you can see whether you already own a game. When no
copy has the code, it asks [UPCitemdb](https://www.upcitemdb.com) for the
product name, then lets you pick the game and add the copy with the barcode,
platform and region filled in. UPCitemdb allows 100 lookups a day without an
account. With `external_services` off, only your own copies are checked, and
viewers only ever check your copies, so they cannot use up the daily lookups.
If the photo cannot be read, type the digits under the bars instead.

## Photos of copies

To keep photos of a copy, such as the box, the cartridge or the manual,
choose **Add photos** from the copy's **⋯** menu on the game page. On a
phone this offers the camera. Each copy takes up to 20 JPEG, PNG or WebP
photos of up to 10 MB and 100 megapixels each. They are stored with the
app's data, so Home Assistant backups include them. Memory Card removes the
location, camera details, thumbnails and other metadata a phone stores in a
photo, and keeps only which way up it goes and its color profile. Uploaded
covers are cleaned the same way.

## Bundles

A bundle is one copy that holds several games, such as a collection on a
single cartridge. Add the bundle as a game with its copy, then use **Add
games inside this copy** on that copy. When the bundle is linked to IGDB,
Memory Card offers the games IGDB lists in it.

Each game inside gets its own copy, so it shows in the library, the backlog
and duplicates. Those copies follow the bundle copy: change the bundle's
platform, region, format, device, account, store, condition or wishlist
status and they change too. Price, estimated value and purchase date stay on the bundle
copy. Counts of copies, on the stats page and in the Home Assistant sensors,
count a bundle once. In the CSV, a copy inside a bundle names the
bundle copy's `copy_id` in its `bundle_copy_id` column.

## Import & export (CSV)

**Import & export** in the sidebar downloads your collection as a CSV file
and imports one back. Its **Hardware** tab does the same for your hardware,
in a file of its own. Start from **Download blank template** to enter a
collection in a spreadsheet. **Export CSV** on the Library page downloads
only the copies that match its filters.

After an import, games without an IGDB link can be linked with
**Cleanup > IGDB > Sync with IGDB**, which matches them by exact title. **Review
close matches** next to it goes through the rest: it shows IGDB's closest
names for each game, to link with a click or mark as not on IGDB.

- One row per copy you own. A row with only `game_title` (and optionally
  `igdb_id` and `play_status`) adds a game with no copies.
- Column names are not case sensitive and can be in any order. Only
  `game_title` is required in every row; rows that add a copy also need
  `platform`, `region` and `format`.
- Files from other spreadsheets and collection apps work too: common
  headers such as `Title`, `Name`, `System`, `Console`, `Serial`, `UPC`,
  `Price Paid`, `Purchase Price`, `Collection Status` or `Comments` are
  recognized, and so are play statuses other trackers use, such as
  `Unplayed`, `Unfinished`, `Beaten` or `Retired`. The preview shows how each header was read; pick another
  column for any it read wrong, or leave one out.
- Platform, device, account and store names match what you already have,
  ignoring case. Common platforms and the digital stores (Nintendo eShop,
  PlayStation Store, Microsoft Store, Steam, Epic Games Store, GOG, itch.io)
  exist from the start. Names that match nothing are created; a new device
  is a console, which you can change on the Hardware page.
- The export fills `copy_id`. Keep it to update that copy when importing the
  file again; leave it empty to add a new copy. Columns the file does not
  have keep that copy's values, while an empty cell clears one. Importing an
  unchanged export changes nothing.
- The import shows a preview first. Rows with errors block the import until
  you fix them in the spreadsheet and choose the file again.
- Save the file as CSV UTF-8. Files that use `;` as separator work too.

| Column | Accepted values |
|---|---|
| `copy_id` | Number from the export, or empty |
| `status` | `owned` or `wanted` (empty means owned). `In Collection` is read as `owned`, `Wish List` and `On Order` as `wanted`. |
| `igdb_id` | IGDB game id, optional |
| `game_title` | Required |
| `play_status` | Your own: `backlog`, `playing`, `paused`, `completed`, `dropped`, `mastered`, or the name or key of a status you added. Exports write your key; everyone else's play stays out of the file. |
| `labels` | Label names, up to 50, separated by commas or semicolons. The import adds them to the game and creates the ones you do not have yet, in gray; it never removes a label. Exports write them separated by `; `. |
| `platform` | Platform name, abbreviation or slug |
| `region` | `na`, `eu`, `au`, `nz`, `jp`, `cn`, `asia`, `kr`, `br`, `ww` (worldwide or region-free); also `NTSC-U`, `PAL`, `NTSC-J` |
| `release_title` | Title on this release, when it differs |
| `product_code` | Serial or product code |
| `format` | `physical` or `digital` |
| `edition` | Free text, such as `Collector's Edition` |
| `device` | Console, handheld or PC the copy is on |
| `account` | Digital only: the store account that owns it |
| `store` | Store name |
| `condition` | Physical only: `sealed`, `cib`, `no_manual`, `loose` |
| `purchased_on` | Date bought, `YYYY-MM-DD` |
| `price` | Price paid, or expected for a wanted copy. `59.99`, `59,99` and `1.299,90` all work; the export writes `1299.90`. Needs `currency`. |
| `currency` | Three-letter ISO 4217 code, such as `USD`, `EUR` or `BRL` |
| `estimated_value` | What an owned copy would sell for now, read like `price`. Uses `estimated_currency`, or `currency` when that is empty. |
| `estimated_currency` | Three-letter code of `estimated_value`, when it differs from `currency` |
| `barcode` | UPC or EAN digits from the box |
| `notes` | Free text |
| `bundle_copy_id` | `copy_id` of the bundle copy this copy came in. Its platform, region, format, device, account, store, condition and status then follow that copy, and its own price, estimated value and purchase date are left out. Empty takes the copy out of a bundle. |

## Stats

![Stats in the dark theme](https://ha-memory-card.pages.dev/images/stats.webp)

_Game titles belong to their respective owners._

**Stats** in the sidebar sums up the collection and your own play: how many
games, copies and wishlist copies the house has, your backlog progress and
your games by play status, your most played games and a calendar of your
play, copies by platform and by store, and what was paid next to what the
copies are worth now. A **Hardware** group does the same for your hardware.

![The Household page comparing everyone's play this month](https://ha-memory-card.pages.dev/images/household.webp)

_Game titles and cover art belong to their respective owners._

**Household** in the sidebar puts everyone's play side by side for this
month, this year or all time: play time, sessions, days played, games
played, games finished and each person's most played game, the most play
time first.

## Home Assistant sensors

Memory Card can add a **Memory Card** device to Home Assistant with the
sensors below, for the household and for each person. Pick one of two ways
with the `sensors` option and restart Memory Card. The Status section under Settings in Memory Card says
whether the sensors are connected.

- **`integration`**: the Memory Card integration, installed from the
  Marketplace. No broker is needed.
  1. Add the
     [Memory Card integration](https://my.home-assistant.io/redirect/marketplace_repository/?owner=claudiosanches&repository=ha-memory-card-integration&category=integration)
     to the Marketplace as a custom repository, which needs the Marketplace
     connected to GitHub. Download it and restart Home Assistant.
  2. Set `sensors` to `integration` and restart Memory Card.
  3. Under **Settings > Devices & services**, Home Assistant shows Memory
     Card as discovered. Select **Add**.
- **`mqtt`**: MQTT discovery. It needs the **Mosquitto broker** app and the
  **MQTT** integration, which Home Assistant offers to set up once the broker
  runs. Start Memory Card after the broker so it finds it.

Both create the same sensors with the same IDs. Switching from one to the
other removes the old ones on the next start, so history graphs carry on.

The household's:

| Entity | State | Attributes |
|---|---|---|
| `sensor.memory_card_games` | Games with at least one owned copy | |
| `sensor.memory_card_copies` | Owned copies | |
| `sensor.memory_card_wishlist` | Copies on the wishlist | |
| `sensor.memory_card_playing` | Owned games someone is playing | `games`: up to 50 titles, the latest played first |
| `sensor.memory_card_last_added` | Title of the game whose owned copy was added last. A wishlist copy counts from when you mark it Got it. | `added_at`, `platform` |
| `sensor.memory_card_pick` | Title of the latest random pick, from Memory Card or Home Assistant | `suggested_at`, `platforms` of its owned copies, `person` whose pick it is |

Each person with a play status or a session gets their own sensors, up to
16, named after them, such as `sensor.memory_card_ana_backlog`. A person
Memory Card has no name for, such as one known only from an NFC reader, is
named by their user ID instead:

| Entity | State | Attributes |
|---|---|---|
| `sensor.memory_card_<name>_backlog` | Their owned games in a status that counts as backlog | |
| `sensor.memory_card_<name>_playing` | Their owned games in a status that counts as playing | `games`: up to 10 titles, the latest played first |
| `sensor.memory_card_<name>_last_played` | Title of the game they last played | `date`, `platform` of the copy played |

A person's sensors keep their entity IDs when they change their name in Home
Assistant.

A pick comes from one person's backlog, so a roll names whose:

- with the integration, the **Roll a pick** button
  (`button.memory_card_roll_pick`) rolls for whoever presses it. The
  `memory_card.roll_pick` action rolls for whoever calls it, and an
  automation names the person with `person: person.ana`. Filters narrow it,
  for example a game for two on the Switch with `device: [switch]` and
  `min_local_players: 2`. It responds with the picked game's `title` and
  `platforms`.
- with MQTT, which cannot tell who sent a message, the `mqtt.publish` action
  sends `{"person": "<user ID>", "device": ["switch"], "min_local_players": 2}`
  to `memory_card/pick/roll`. A message without a person is ignored, and
  rolls come once a second at most.

Rolls need managers set in the options, and the person must be a manager or
a viewer with `viewers_track_progress` on. A person rolls only for
themselves. A roll that names a person, from an automation or MQTT, needs
someone Memory Card knows: a manager, or a person who has already changed
something in it, such as their play status.

The fields match the Backlog page's picker: `play_status` (a list of play
status keys), `platform`, `device`, `genre`, `game_mode` (lists of slugs),
`min_local_players` and `max_hours_to_beat` (hours for the main story). Each platform, device and store shows its slug in
its Manage form, and library links use the same slugs.

The counts keep long-term statistics, so history graphs show how the
collection and each backlog change. Sensors update a second after each
change and show as unavailable while Memory Card is stopped. Setting the `sensors` option back
to `off` removes the device on the next start.

## NFC tags

Put an NFC tag on a game's box, and scanning it with the Home Assistant
companion app starts that copy for you: Memory Card logs a play session for
today in your play and sets your status for the game to playing. To link a
tag, choose **Link NFC tag** from the copy's **⋯** menu on the game page,
then scan the tag within a minute. For
the Home Assistant app, write a blank tag first under **Settings >
Connectivity > Tags > Add tag**. ESPHome readers read any tag as it is.

A scan logs play for managers, and for viewers when `viewers_track_progress`
is on; nothing is logged while no managers are set. A reader with no Home
Assistant user, such as an ESPHome NFC reader, logs play for the person the
`tag_readers` option names for it, and nothing otherwise:

```yaml
tag_readers:
  - device: 4f1c9e0d8a7b6c5d4e3f2a1b0c9d8e7f
    person: your-user-id
```

`device` is the reader's Home Assistant device ID: open the reader under
**Settings > Devices & services > Devices**, and it is the last part of the
page's address. When a reader no option names scans a linked tag, the app's
**Log** tab shows its device ID. `person` is a user ID from **Settings >
Status** in Memory Card, as for `managers`.

A link waits for a scan by the manager who started it, or by a reader with no
user that `tag_readers` does not give to someone else. Pages open in Memory
Card show a scan within a second, like any change someone else makes. Scanning the same tag again on the same day does not add
another session for that person. A game whose status counts as finished goes
back to playing too, unless the `tag_scan_finished` option is set to `keep`.
Memory Card hears scans through the Home Assistant API, which is why the app
asks for access to it; the Status section under Settings says whether it is
listening. **Unlink NFC tags** in the same menu removes a copy's tags.

## Language and currency

Memory Card comes in English, Spanish and Brazilian Portuguese. It follows
Home Assistant's language, or your browser's when Home Assistant uses one
the app does not have. To pick one, go to **Settings > Appearance >
Language**; the choice is saved in that browser. The app's options in Home
Assistant are translated too.

Dates follow the language, so English writes 11/23/2026. To write them
another way in every language, set the `date_format` option.

New prices start in the `currency` option when it is set, else in the
currency of the copy priced last, else in Home Assistant's currency.

Memory Card reads Home Assistant's language and currency once after it
starts. Restart the app after changing either of them in Home Assistant.

## Themes

Memory Card comes in 12 themes, each with a light and a dark version. To
pick one, go to **Settings > Appearance > Theme**. The button at the top
right switches between light, dark and your system's setting, and so does
**Color scheme** under **Settings > Appearance**. Both choices are saved in
that browser, so each person and each device keeps their own.

The first five keep to quiet colors and let the covers stand out. The rest
give the whole app a style of its own.

| Theme | Look |
| --- | --- |
| Default | Cool slate with teal accents, blue in the dark. |
| Monochrome | Black, white and grays with no tint, so the covers bring the color. |
| Warm Stone | Cream and travertine with espresso and sand accents. |
| Slate | Low-glare slate and frost blue, easy on the eyes for long browsing. |
| Sage | Pale mist, or forest charcoal in the dark, with eucalyptus accents. |
| Retro | An 80s console: parchment or charcoal, faint scanlines, orange and gold stripes and a pixel-art gamepad. |
| 2000's Internet | A glossy game portal: enamel blue headers, glass buttons, a starfield and a spinning globe. |
| Steampunk | Vellum and mahogany, serif headings, brass controls, ledger tabs and gears. |
| Dieselpunk | 1930s Art Deco: tread-plate steel, sharp corners, uppercase headings, amber accents and a pressure gauge. |
| Cyberpunk | Neon magenta, violet and cyan over a grid, with a city skyline. |
| Circuit | Monospace wireframe: a green dot matrix in light, a phosphor CRT in dark, and a microchip. |
| Pastel Plush | Soft and rounded: strawberry cream with a bunny in light, berry plum with a teddy bear in dark. |

## Data and backups

Everything the app stores lives in the app's data folder, which Home Assistant
includes in its backups:

- `db/memory-card.sqlite`: your collection.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded covers, and photos of copies and
  hardware.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.

## Support

Questions, ideas and bug reports go to
[GitHub issues](https://github.com/claudiosanches/ha-memory-card/issues).
For a bug, say which version of Memory Card you run and add the lines around
the problem from the app's **Log** tab.

To support development, use
[GitHub Sponsors](https://github.com/sponsors/claudiosanches) or
[Ko-fi](https://ko-fi.com/claudiosanches).
