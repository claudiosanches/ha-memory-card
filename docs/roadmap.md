# Roadmap

Ideas for the next versions of Memory Card. None has a date yet, and the order
can change with what people ask for.

## Ideas

### Steam library import

Bring your Steam games in as digital copies on your Steam account, each matched
to its IGDB game by its Steam ID, with no title guessing. It uses Steam's
official Web API with your own Steam ID and API key.

### Play from Home Assistant's gaming integrations

Home Assistant's PlayStation Network, Steam and Xbox integrations already show
what someone is playing. Each of those entities would be mapped to a person, as
`tag_readers` maps an NFC reader, and Memory Card would log a play session when
a game from the library shows up.

### Loans

Note who borrowed a copy and since when, then mark it returned. A Home Assistant
sensor could list the games lent out.

### Ratings and reviews

A rating and a short review from each person, side by side on the game page.

### Year in review

The household's year in games: what each person finished, hours played and the
most played games.

### Upcoming releases

Release dates of games on the wishlist, from IGDB, as a Home Assistant calendar.

### QR shelf labels

Printable QR labels for the shelf that open a game's page. NFC tags stay the way
to log play.

## Official APIs only

Memory Card uses only official APIs and Home Assistant's own integrations.
Stores with no official way to read a library, such as Nintendo, PlayStation and
GOG, stay with [CSV import](./docs/import-export.md),
[barcode scans](./docs/barcodes.md) or adding games by hand.

## Support development

Memory Card is free and open source. If you'd like to see these sooner, you can
support its development on
[GitHub Sponsors](https://github.com/sponsors/claudiosanches) or
[Ko-fi](https://ko-fi.com/claudiosanches).

Ideas and requests are welcome on
[GitHub issues](https://github.com/claudiosanches/ha-memory-card/issues).
