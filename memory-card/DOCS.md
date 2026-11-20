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

## Data and backups

Everything the app stores lives in the app's data folder, which Home Assistant
includes in its backups:

- `db/memory-card.sqlite`: your collection.
- `db/pre-migration/`: a copy of the database taken before each upgrade that
  changes its structure. The last five are kept.
- `covers/`: downloaded and uploaded cover images.

Backups stop the app for a moment so the database is copied in a consistent
state. The app starts again on its own when the backup finishes.
