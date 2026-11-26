# Getting started

Memory Card needs Home Assistant 2026.11 or newer.

## Install the app

[![Open your Home Assistant instance and show the add app repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card)

1. Select the button above to add the app's repository to Home Assistant. Or
   go to **Settings > Apps > Install app**, open the ⋮ menu, choose
   **Repositories** and add `https://github.com/claudiosanches/ha-memory-card`.
2. Find **Memory Card** in the list, select it and choose **Install**.
3. Start the app. **Memory Card** shows in the sidebar.

## How a household uses it

Memory Card keeps one collection for the whole household, and each person's
play apart.

- The collection is shared: games, copies, devices, stores, platforms,
  labels, the list of play statuses, NFC tags, photos and covers. Everyone
  can browse it, and managers change it.
- Play is each person's own: their play status for every game, which makes
  their backlog, their play sessions and their picks. Only they change it,
  managers included.
- A person is a Home Assistant user. Home Assistant signs you in, so Memory
  Card has no login of its own.

## Name the managers

A new install is read-only for everyone until it has a manager.

1. Open Memory Card as the person who will manage the collection.
2. Go to **Settings > Status**. Under **Your access**, select **Copy user ID**.
3. In Home Assistant, go to **Settings > Apps > Memory Card** and open the
   **Configuration** tab.
4. Paste the ID under **Managers** and save. Add one entry for each manager.
   If the form shows no field for it, choose **Edit in YAML** from the ⋮ menu
   and write:

   ```yaml
   managers:
     - your-user-id
   ```

5. Restart the app from the **Info** tab.

Back in Memory Card, **Your access** now says you can manage the collection.

Everyone else is a viewer: they browse the collection, but do not change it.
To let viewers keep their own play status, sessions and picks, turn on
**viewers_track_progress** in the same **Configuration** tab and restart the
app.

## Connect IGDB

[IGDB](https://www.igdb.com) gives Memory Card titles, covers, release dates,
genres and other details. It is optional: without it you enter games by hand.
IGDB uses Twitch accounts:

1. Turn on two-factor authentication on your Twitch account. Twitch requires
   it to register applications.
2. Open the [Twitch developer console](https://dev.twitch.tv/console/apps)
   and register an application:
   - **OAuth Redirect URLs:** `http://localhost`
   - **Category:** Application Integration
   - **Client Type:** Confidential
3. Open the application with **Manage**, copy the **Client ID** and select
   **New Secret** to make a **Client Secret**.
4. In Home Assistant, open the app's **Configuration** tab, fill in
   **igdb_client_id** and **igdb_client_secret**, and save. To keep them in
   Home Assistant's `secrets.yaml` instead, enter `!secret` and the key, such
   as `!secret igdb_client_secret`.
5. Restart the app.

**Settings > Status** in Memory Card shows **IGDB: Connected** once it works. See
[IGDB](igdb.md) for what it fills in.

## Add your first games

There are three ways in:

- **One at a time:** select **Add game** on the [Library](library.md), type
  the title and pick the match from IGDB. Then add your copies on the
  [game page](game.md).
- **From a spreadsheet:** go to [Import & export](import-export.md), start
  from **Download blank template** or use a file you already keep, and
  import it.
- **From the box:** select **Scan barcode** and point a phone at the box. See
  [Barcodes](barcodes.md).

Before adding many copies, add your consoles and accounts under
[Hardware](hardware.md), so each copy can say which device it lives on.

## Next

- Set your play status for the games you own, and Memory Card builds your
  [Backlog](backlog.md).
- Add Memory Card's [sensors](sensors.md) to Home Assistant dashboards.
- Pick a [theme](themes.md).
