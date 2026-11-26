# Manage

**Manage** in the sidebar holds the lists the collection is built from. Only
managers see it.

![Manage, on the Platforms tab](https://ha-memory-card.pages.dev/images/manage.webp)

The store, platform and label forms, and the device form on
[Hardware](hardware.md), show each one's slug: the short name that library
links, Home Assistant rolls and CSV files use.

Consoles, handhelds, PCs and accounts are on the [Hardware](hardware.md)
page, with the rest of your hardware.

## Stores

The shops and online stores you buy games from. **Add store** asks for a
name. The digital stores exist from the start. Deleting a store keeps its
copies, with no store.

## Platforms

**Your platforms** are the ones offered when adding games and devices, in the
order shown. Drag to reorder them.

**All platforms** lists every platform Memory Card knows, by type: consoles,
handhelds, computers, mobile and arcade. **Add** puts one in your platforms.
Removing one hides it when adding games and devices; its games and devices
stay. **Add custom platform** adds one that is not on the list.

## Play statuses

The statuses everyone picks from for their own play, in the order shown. Drag
to reorder them.

The six built-in statuses (backlog, playing, paused, completed, dropped and
mastered) can be renamed, recolored and reordered, but not deleted. Their keys
never change, so CSV files and Home Assistant can rely on them.

**Add status** asks for a name, a color and what it counts as, which decides
where its games show on the [Backlog](backlog.md):

- **Backlog:** waiting, and picked by **Pick for me**.
- **Playing** and **On hold:** in those sections.
- **Finished:** counted as cleared.
- **Not counted:** left out of the Backlog page.

Deleting a status asks where to move everyone's games in it, or leaves them
with none.

## Labels

Your own groups for games, such as "Couch co-op" or "Replay someday". **Add
label** asks for a name and a color, and the list shows how many games have
each. Set a game's labels with **Edit** on its [page](game.md). Names cannot
contain commas or semicolons, which separate labels in CSV files. Deleting a
label keeps its games.
