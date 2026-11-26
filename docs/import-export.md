# Import & export

**Import & export** in the sidebar moves the collection between Memory Card
and a spreadsheet as a CSV file. Only managers see it.

![Importing a file from another spreadsheet](https://ha-memory-card.pages.dev/images/import.webp)

## Export

- **Export collection** downloads every copy, one per row.
- **Download blank template** downloads the columns with no rows, to fill in
  a spreadsheet.
- **Export CSV** on the [Library](library.md) downloads only the copies that
  match its filters.

The `play_status` column holds your own status. Everyone else's play stays out
of the file.

## Import

1. Save the spreadsheet as CSV UTF-8. Files that use `;` as separator work
   too.
2. Under **CSV file**, choose the file.
3. **How your columns are read** shows which column each header went to.
   Memory Card knows its own names and common ones from other spreadsheets
   and collection apps, such as `Title`, `Name`, `System`, `Console`,
   `Serial`, `UPC`, `Price Paid`, `Purchase Price`, `Collection Status` or
   `Comments`. Pick another column for a header it read wrong, or **Not
   imported** to leave one out.
4. Check the preview: new and updated games and copies, and the platforms,
   devices, stores and labels it will create.
5. Rows with errors block the import. Fix them in the spreadsheet and **Choose
   another file**. Rows with warnings import as they are.
6. Select **Import**. Nothing changes until you do.

After an import, **Review IGDB matches** goes through the games that are not
linked to IGDB yet. See [IGDB](igdb.md#close-matches).

## How rows are read

- One row per copy. A row with only `game_title`, and optionally `igdb_id`
  and `play_status`, adds a game with no copies.
- Column names are not case sensitive and can be in any order. Only
  `game_title` is required in every row; rows that add a copy also need
  `platform`, `region` and `format`.
- Platform, device, account and store names match the ones you have,
  ignoring case. Common platforms and the digital stores (Nintendo eShop,
  PlayStation Store, Microsoft Store, Steam, Epic Games Store, GOG, itch.io)
  exist from the start. Names that match nothing are created: a new device
  is a console, which you can change under [Hardware](hardware.md), and a
  new account shows under [Manage > Accounts](manage.md#accounts). A row
  cannot put a copy on a controller, accessory or toy.
- The export fills `copy_id`. Keep it to update that copy when importing the
  file again; leave it empty to add a new copy. Columns the file does not
  have keep that copy's values, while an empty cell clears one. Importing an
  unchanged export changes nothing.
- An import never removes a copy, a game or a label.

## Columns

| Column | Accepted values |
|---|---|
| `copy_id` | Number from the export, or empty |
| `status` | `owned` or `wanted` (empty means owned). `In Collection` is read as `owned`, `Wish List` and `On Order` as `wanted`. |
| `igdb_id` | IGDB game id, optional. Tells apart games with the same title. |
| `game_title` | Required |
| `play_status` | Your own: `backlog`, `playing`, `paused`, `completed`, `dropped`, `mastered`, or the name or key of a status a manager added. Words other trackers use work too, such as `Unplayed`, `Unfinished`, `Beaten` or `Retired`. |
| `labels` | Label names, up to 50, separated by commas or semicolons. The import adds them to the game and creates the ones that do not exist yet, in gray. Exports write them separated by `; `. |
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
| `bundle_copy_id` | `copy_id` of the bundle copy this copy came in. Its platform, region, format, device, store, condition and status then follow that copy, and its own price, estimated value and purchase date are left out. Empty takes the copy out of a bundle. |

**Columns and accepted values** at the bottom of the page lists the same in
the app.

The export escapes cells that a spreadsheet would read as a formula, so
opening it is safe.

## Hardware

The **Hardware** tab moves your [hardware](hardware.md) to and from its own
CSV file. Accounts are not in it; they come with the games' file. **Export hardware** downloads every item, and **Download blank
template** the columns alone.

An import shows a preview first. Each row updates the hardware with the same
`device_id`, or else the same name, and adds new hardware otherwise. Columns
the file does not have keep their values, while an empty cell clears one.
Stores that do not exist yet are added. Rows with errors block the import.

| Column | Accepted values |
|---|---|
| `device_id` | Number from the export, or empty |
| `status` | `owned` or `wanted` (empty means owned) |
| `name` | Required |
| `kind` | `console`, `handheld`, `pc`, `controller`, `accessory` or `toy`. Required for new hardware. |
| `model` | Free text |
| `platform` | What a controller, accessory or toy is for: platform name, abbreviation or slug |
| `plays` | What a console, handheld or PC plays: platforms separated by `;` |
| `region` | As for copies |
| `condition` | `sealed`, `cib`, `no_manual`, `loose` |
| `store` | Store name |
| `purchased_on` | `YYYY-MM-DD` |
| `price`, `currency` | Price paid, or expected for wanted hardware, read as for copies |
| `estimated_value`, `estimated_currency` | What it would sell for now |
| `serial_number` | Free text |
| `barcode` | UPC or EAN digits |
| `notes` | Free text |
| `active` | `yes` or `no` |
