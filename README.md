# Memory Card

[![GitHub Release][releases-shield]][releases]
![Project Stage][project-stage-shield]
[![License][license-shield]](LICENSE)

[![GitHub Actions][github-actions-shield]][github-actions]
![Project Maintenance][maintenance-shield]

A video game collection manager that runs as a Home Assistant app. It tracks
the games you own across consoles, the platform and region releases of each
game, and every copy you have, physical or digital.

![The library in Memory Card](https://ha-memory-card.pages.dev/images/library.webp)

_Game titles and cover art belong to their respective owners._

## Install

Memory Card needs Home Assistant 2026.11 or newer.

[![Open your Home Assistant instance and show the add app repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card)

1. Select the button above to add the app's repository to Home Assistant. Or
   go to **Settings > Apps > Install app**, open the ⋮ menu, choose
   **Repositories** and add `https://github.com/claudiosanches/ha-memory-card`.
2. Install **Memory Card** and start it. It appears in the sidebar.

[Getting started](docs/getting-started.md) goes through the setup step by
step. Questions and bug reports go to
[GitHub issues](https://github.com/claudiosanches/ha-memory-card/issues).

## Documentation

- [The guide](docs/README.md): every page and feature, with screenshots, and
  the [themes](docs/themes.md).
- [Options](memory-card/DOCS.md): the reference Home Assistant shows in the
  app's **Documentation** tab.
- [Development](docs/development.md), [Local Home Assistant](docs/local-home-assistant.md)
  and [Releasing](docs/releasing.md): building, testing and publishing from
  this repository.

## Credits

- Interface icons from [Tabler Icons](https://tabler.io/icons) (MIT).
- Game metadata and covers from [IGDB](https://www.igdb.com), when IGDB credentials are set.
- Product names for scanned barcodes from [UPCitemdb](https://www.upcitemdb.com).

Game titles, cover art and other trademarks shown in the screenshots belong to
their respective owners. Memory Card is not affiliated with or endorsed by them.

## License

Memory Card is licensed under the Apache License, version 2.0. See [LICENSE](LICENSE).

[releases-shield]: https://img.shields.io/github/v/release/claudiosanches/ha-memory-card.svg
[releases]: https://github.com/claudiosanches/ha-memory-card/releases
[project-stage-shield]: https://img.shields.io/badge/project%20stage-production%20ready-brightgreen.svg
[license-shield]: https://img.shields.io/github/license/claudiosanches/ha-memory-card.svg
[github-actions-shield]: https://github.com/claudiosanches/ha-memory-card/actions/workflows/ci.yml/badge.svg
[github-actions]: https://github.com/claudiosanches/ha-memory-card/actions/workflows/ci.yml
[maintenance-shield]: https://img.shields.io/maintenance/yes/2026.svg
