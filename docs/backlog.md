# Backlog

**Backlog** in the sidebar shows your games by your play status: what you are
playing, what is on hold and what is waiting for you. It is yours; everyone in
the house has their own. The bar at the top counts how many you have cleared.

![The backlog with a random pick](https://ha-memory-card.pages.dev/images/backlog.webp)

_Game titles and cover art belong to their respective owners._

## Sort your games

Games you have given no play status wait under **Not sorted yet**. **Sort
them** goes through them one at a time: pick a status, or **Skip** to leave
one for later, then **Done**.

Logging a session for a game in your backlog, or with no status, sets it to
playing. **Played today** on the [game page](game.md) does the same.

## Pick for me

**Can't decide?** rolls a random game from your backlog.

1. Under **Play on**, keep **Any device** or choose the device you want to
   play on.
2. Select **Pick for me**. A game picked for you in the last day comes up
   again only when nothing else is left.
3. **Start playing** sets the game to playing. **Pick another** rolls again.

The latest pick also shows in Home Assistant's `sensor.memory_card_pick`, and
Home Assistant can roll one for you. See [sensors](sensors.md).

## Sections

- **Now playing:** **Completed**, **Mastered**, **Pause** and **Drop** move a
  game on.
- **On hold:** **Resume** puts a game back to playing.
- **In the backlog:** **Start** begins one.

Which section a status lands in depends on the group it counts as, which
managers set under **Manage > Play statuses**:

| Counts as | On the Backlog page |
| --- | --- |
| Backlog | In the backlog, and picked by **Pick for me** |
| Playing | Now playing |
| On hold | On hold |
| Finished | Counted as cleared |
| Not counted | Left out |

## Viewers

Managers always keep their own backlog. A viewer sees theirs once a manager
turns on `viewers_track_progress` in the app's options.
