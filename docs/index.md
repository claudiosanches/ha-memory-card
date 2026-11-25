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
      text: Get started
      link: /docs/getting-started
    - theme: alt
      text: Read the guide
      link: /docs/
    - theme: alt
      text: GitHub
      link: https://github.com/claudiosanches/ha-memory-card

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

1. In Home Assistant, go to **Settings > Apps > Install app**.
2. Open the ⋮ menu, choose **Repositories** and add
   `https://github.com/claudiosanches/ha-memory-card`.
3. Install **Memory Card** and start it. It appears in the sidebar.

[Getting started](/docs/getting-started) goes through the setup step by step.

## Support development

Memory Card is free and open source. If it is useful to your household, you
can support its development on
[GitHub Sponsors](https://github.com/sponsors/claudiosanches) or
[Ko-fi](https://ko-fi.com/claudiosanches). [About](/about) has more on the
project.
