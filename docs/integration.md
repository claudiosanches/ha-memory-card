# Memory Card integration

The Memory Card integration brings the app's [sensors](sensors.md) and a
**Roll a pick** button into Home Assistant, plus a `memory_card.roll_pick`
action for automations and scripts. It needs no MQTT broker.

The app does the work: it counts, picks and tells the integration what
changed. The integration creates the same sensors, with the same IDs, as the
app's MQTT mode, so you can switch between the two without breaking
dashboards. The **Roll a pick** button comes only with the integration.

## Integration or MQTT

| | Integration | MQTT |
|---|---|---|
| Needs | The integration from the Marketplace | The Mosquitto broker app and the MQTT integration |
| Sensors | The same | The same |
| **Roll a pick** button | Rolls for whoever presses it | None |
| Rolls from automations | `memory_card.roll_pick`, with a response | `mqtt.publish` with the person's user ID, no response |

Choose the integration unless you already run Mosquitto for other devices and
want everything on it.

## Requirements

Home Assistant 2026.11 or newer, with the Memory Card app 0.1.0 or newer.

## Install

1. In Home Assistant, go to **Settings > Marketplace**, search for
   **Memory Card** and open it. Select **Install**, then **Install** again in
   the dialog, and restart Home Assistant if it asks. If the integration's
   setup opens right away, it says Memory Card sets itself up: close it and go
   on.
2. In Home Assistant, go to **Settings > Apps > Memory Card** and open the
   **Configuration** tab. Set **Home Assistant sensors** (`sensors`) to
   `integration`, save, and restart the app from the **Info** tab.
3. Go to **Settings > Devices & services**. Memory Card shows under
   **Discovered**. Select **Add**, then **Submit**.

There is nothing to type: the app tells Home Assistant where to find it and
which token to use. **Settings > Status** in Memory Card shows **Home
Assistant sensors: Connected** once it works.

Adding Memory Card by hand from **Add integration** stops with a message that
it sets itself up: use the app's option instead.

This button opens Memory Card in your Marketplace. A Marketplace that does not
list it yet offers to add
[the integration's repository](https://github.com/claudiosanches/ha-memory-card-integration)
as a custom repository, which needs the Marketplace connected to GitHub.

[![Open your Home Assistant instance and open a repository in the Marketplace.](https://my.home-assistant.io/badges/marketplace_repository.svg)](https://my.home-assistant.io/redirect/marketplace_repository/?owner=claudiosanches&repository=ha-memory-card-integration&category=integration)

## What it adds

A **Memory Card** device with:

- the household's sensors: games, copies, wishlist, playing, last added and
  the current pick;
- three sensors for each person who keeps play in Memory Card, up to 16:
  their backlog, what they are playing and what they last played;
- the **Roll a pick** button.

[Home Assistant sensors](sensors.md) lists every entity with its state and
attributes. Values update about a second after each change in Memory Card,
and the entities show as unavailable while the app is stopped. A person's
sensors come and go as people start or stop keeping play.

Entity names come in English, Spanish and Brazilian Portuguese, following
Home Assistant's language.

## Roll a pick button

**Roll a pick** (`button.memory_card_roll_pick`) picks a random owned game
from the backlog of whoever presses it, as **Pick for me** on the
[Backlog](backlog.md) does. The pick shows in `sensor.memory_card_pick`,
with whose it is in its `person` attribute.

A dashboard card with both:

```yaml
type: entities
title: What to play
entities:
  - entity: sensor.memory_card_pick
  - entity: button.memory_card_roll_pick
```

A **Button** card can press it too: set its tap behavior to **Perform
action**, pick **Press button** and choose **Roll a pick** as the target.

![A Button card that presses Roll a pick](https://ha-memory-card.pages.dev/images/integration-roll-button.webp)

The same card in YAML:

```yaml
type: button
name: Roll a pick
icon: mdi:dice-5
show_state: false
tap_action:
  action: perform-action
  perform_action: button.press
  target:
    entity_id: button.memory_card_roll_pick
```

An automation pressing the button has no person, so it is refused. Use the
action below instead.

## The roll_pick action {#the-roll_pick-action}

`memory_card.roll_pick` picks a random owned game from one person's backlog
and responds with its `title` and `platforms`.

| Field | What it does |
|---|---|
| `person` | Whose backlog, as a `person` entity. Leave it out to roll for yourself. An automation must name someone. |
| `play_status` | Play status keys to pick from. Empty means every status that counts as backlog. |
| `platform` | Platform slugs. |
| `device` | Device slugs: copies on these devices, or with no device on a platform one of them plays. |
| `genre` | Genre slugs. |
| `game_mode` | Game mode slugs. |
| `min_local_players` | At least this many players on the same screen. |
| `max_hours_to_beat` | At most this many hours for the main story, from IGDB. Games without a time are left out. |

The slug fields take one slug or a list of up to 10. Each platform shows its
slug in its [Manage](manage.md) form and each device in its
[Hardware](hardware.md) form.

A game night pick for two on the Switch, from Ana's backlog:

```yaml
actions:
  - action: memory_card.roll_pick
    data:
      person: person.ana
      device: switch
      min_local_players: 2
    response_variable: pick
  - action: persistent_notification.create
    data:
      title: Game night
      message: "{{ pick.title }} on {{ pick.platforms | join(', ') }}"
```

Who can roll:

- The app needs managers, and the person must be a manager or a viewer the
  app lets keep play (`viewers_track_progress`).
- A person rolls only for themselves. A script someone runs from a dashboard
  counts as that person, so it cannot name someone else.
- An automation has no person of its own, so it must name one, and Memory
  Card must know them: a manager, or someone who has already changed
  something in it, such as their play status.
- The `person` entity must be linked to a Home Assistant user.

## Errors

| Message | What to do |
|---|---|
| Name a person to roll a pick for. | Add `person`, or roll from a dashboard as yourself. |
| You can only roll a pick for yourself. | Leave `person` out, or name yourself. |
| Memory Card does not keep play for this person. | Make them a manager, or turn on `viewers_track_progress`. A person Memory Card has never seen needs to open it and set a play status first. |
| This person has no Home Assistant user. | Link the person to a user under **Settings > People**. |
| No game matches these filters. | Loosen the filters, or add games to the backlog. |
| Memory Card could not read these filters. | Check the slugs against the Manage and Hardware forms. |
| Cannot reach the Memory Card app. | Start the app, and check its **Log** tab. |
| Memory Card is not set up. | Add Memory Card from **Discovered** under **Settings > Devices & services**. |

## Troubleshooting

- **Settings > Status** in Memory Card says what the sensors are doing.
  **Waiting for the Memory Card integration** means the app is ready but the
  integration is not added yet: add it from **Discovered**, or install it if
  it is not there.
- The app's **Log** tab says why the feed could not start.
- **Download diagnostics** on the integration's page in **Settings > Devices &
  services** gives what the integration holds from the app, without the
  token, anyone's play or user IDs. Attach it to a bug report.

## Update and remove

Updates come through the Marketplace. Restart Home Assistant after one.

To remove it, set the app's `sensors` option to `off` or `mqtt` and restart
the app: Home Assistant removes the integration's entry and entities. Then
remove **Memory Card** from the Marketplace. With `mqtt`, the same entity IDs
come back over MQTT.

Bugs in the integration go to its
[issues](https://github.com/claudiosanches/ha-memory-card-integration/issues).
