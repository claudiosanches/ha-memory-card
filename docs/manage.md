# Manage

**Manage** in the sidebar holds the lists the collection is built from. Only
managers see it.

![Manage, on the Platforms tab](https://ha-memory-card.pages.dev/images/manage.webp)

The account, store, platform and label forms, and the device form on
[Hardware](hardware.md), show each one's slug: the short name that library
links and Home Assistant rolls use.

Consoles, handhelds and PCs are on the [Hardware](hardware.md) page, with
the rest of your hardware.

## Accounts

The store accounts that own your digital games, such as a Nintendo Account,
PlayStation Network or Steam. A digital copy names its account apart from
the device it is installed on, so a game bought on a PlayStation Network
account and played on the PS5 has both. **Add account** asks for:

- a name, which picks the account for a copy;
- **Store:** where it buys games. A new copy on the account takes it as its
  store;
- **Person:** whose account it is in the household;
- **Plays:** the platforms whose games it holds. Copies on other platforms
  are not offered it.

Inactive accounts are left out when choosing the account of a copy. Deleting
an account keeps its copies, with no account.

## Stores

The shops and online stores you buy games from. **Add store** asks for a
name. The digital stores exist from the start. Deleting a store keeps its
copies, with no store.

## Platforms

**Your platforms** are the ones offered when adding games, devices and
accounts, in the order shown. Drag to reorder them.

**All platforms** lists every platform Memory Card knows, by type: consoles,
handhelds, computers, mobile, arcade and other. **Add** puts one in your
platforms. Removing one hides it when adding games, devices and accounts;
what already uses it stays. **Add custom platform** adds one that is not on
the list. A custom platform that games, devices or accounts use cannot be
deleted; remove it from your platforms instead.

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
