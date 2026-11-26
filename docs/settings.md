# Settings

**Settings** at the bottom of the sidebar has two tabs: **Appearance** and
**Status**. Everyone can open it. **Appearance** opens first once a manager is
configured; on a new install, **Status** opens first so you can copy your user
ID.

![Settings, on the Appearance tab](https://ha-memory-card.pages.dev/images/settings.webp)

## Appearance

Choices here are saved in that browser, so each person and each device keeps
their own.

- **Theme:** one of 12 themes. See [Themes](themes.md).
- **Color scheme:** **System**, **Light** or **Dark**. The button at the top
  right of every page switches it too.
- **Language:** English, Spanish or Brazilian Portuguese. **Automatic** follows
  Home Assistant's language, or the browser's when Memory Card does not have
  Home Assistant's.

Dates follow the language, so English writes 11/23/2026. To write them another
way in every language, set the app's `date_format` option.

## Status

**Integrations** says whether each connection works:

- **IGDB:** connected, not configured, or off because external services are
  turned off. See [IGDB](igdb.md).
- **Home Assistant sensors:** connected, waiting for the integration, or what
  stops them. See [sensors](sensors.md).
- **NFC tag scans:** whether Memory Card hears scans from Home Assistant. See
  [NFC tags](nfc-tags.md).

**Your access** shows who you are signed in as, your role and your Home
Assistant user ID. **Copy user ID** copies it, to name a manager in the app's
options or a person for an NFC reader.

**About Memory Card** shows the version, the license, links to the
documentation and support, and where to support development.
