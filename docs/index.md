---
layout: home

hero:
  name: Memory Card
  text: Your game collection in Home Assistant
  tagline: Track the games your household owns across consoles, physical or digital, and what each person plays.
  image:
    src: /favicon.svg
    alt: Memory Card
  actions:
    - theme: brand
      text: Add to Home Assistant
      link: https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card
    - theme: alt
      text: Get started
      link: /docs/getting-started
    - theme: alt
      text: Read the guide
      link: /docs/

features:
  - title: Every copy you own
    details: Games, their platform and region releases, and each copy, physical or digital, with where it came from and what it cost.
    link: /docs/library
  - title: Covers and details from IGDB
    details: Titles, covers, release dates, genres and player counts filled in for you, and a sync that keeps them current.
    link: /docs/igdb
  - title: Everyone's own play
    details: The household shares the collection. Each person keeps their own status, sessions, backlog and picks.
    link: /docs/household
  - title: Barcodes and NFC tags
    details: Scan a box in a shop to see whether you own it, and tap a tag on a box to log play.
    link: /docs/barcodes
  - title: Home Assistant sensors
    details: Counts, picks and per-person sensors for dashboards and automations, over MQTT or the Memory Card integration.
    link: /docs/sensors
  - title: 13 themes
    details: Each with a light and a dark version.
    link: /docs/themes
---

## A look inside

![The library, with covers from IGDB](images/library.webp)

_Game titles and cover art belong to their respective owners._

![The library and a game page on a phone](images/phone.webp)

_Game titles and cover art belong to their respective owners._

## Install

Memory Card needs Home Assistant 2026.11 or newer.

[![Open your Home Assistant instance and show the add app repository dialog with a specific repository URL pre-filled.](https://my.home-assistant.io/badges/supervisor_add_addon_repository.svg)](https://my.home-assistant.io/redirect/supervisor_add_addon_repository/?repository_url=https%3A%2F%2Fgithub.com%2Fclaudiosanches%2Fha-memory-card)

1. Select the button above to add the app's repository to Home Assistant. Or
   go to **Settings > Apps > Install app**, open the ⋮ menu, choose
   **Repositories** and add `https://github.com/claudiosanches/ha-memory-card`.
2. Install **Memory Card** and start it. It appears in the sidebar.

[Getting started](/docs/getting-started) goes through the setup step by step.

## Support development

Memory Card is free and open source. If it is useful to your household, you
can support its development on
[GitHub Sponsors](https://github.com/sponsors/claudiosanches) or
[Ko-fi](https://ko-fi.com/claudiosanches).
