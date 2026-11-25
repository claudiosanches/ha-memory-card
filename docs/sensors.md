# Home Assistant sensors

Memory Card can add a **Memory Card** device to Home Assistant with sensors
for the household and for each person, for dashboards and automations.

## Set them up

Pick one of two ways with the `sensors` option.

### With the Memory Card integration

No MQTT broker is needed.

1. Add the
   [Memory Card integration](https://my.home-assistant.io/redirect/marketplace_repository/?owner=claudiosanches&repository=ha-memory-card-integration&category=integration)
   to the Marketplace as a custom repository, which needs the Marketplace
   connected to GitHub. Download it and restart Home Assistant.
2. In the app's **Configuration** tab, set `sensors` to `integration`, save
   and restart Memory Card.
3. Under **Settings > Devices & services**, Home Assistant shows Memory Card
   as discovered. Select **Add**.

### With MQTT

1. Install the **Mosquitto broker** app and start it. Home Assistant offers
   to set up the **MQTT** integration once the broker runs.
2. In Memory Card's **Configuration** tab, set `sensors` to `mqtt`, save and
   restart Memory Card. Start it after the broker so it finds it.

Both ways create the same sensors with the same IDs. Switching from one to the
other removes the old ones on the next start, so history graphs carry on.
Setting `sensors` back to `off` removes the device. **Settings > Status** in
Memory Card says whether the sensors are connected.

## The household's sensors

| Entity | State | Attributes |
|---|---|---|
| `sensor.memory_card_games` | Games with at least one owned copy | |
| `sensor.memory_card_copies` | Owned copies | |
| `sensor.memory_card_wishlist` | Copies on the wishlist | |
| `sensor.memory_card_playing` | Owned games someone is playing | `games`: their titles, the latest played first |
| `sensor.memory_card_last_added` | Title of the game whose owned copy was added last | `added_at`, `platform` |
| `sensor.memory_card_pick` | Title of the latest random pick, from Memory Card or Home Assistant | `suggested_at`, `platforms` of its owned copies, `person` whose pick it is |

## Each person's sensors

Each person who keeps play, up to 16, gets their own sensors, named after
them, such as `sensor.memory_card_ana_backlog`:

| Entity | State | Attributes |
|---|---|---|
| `sensor.memory_card_<name>_backlog` | Their owned games in a status that counts as backlog | |
| `sensor.memory_card_<name>_playing` | Their owned games in a status that counts as playing | `games`: up to 10 titles, the latest played first |
| `sensor.memory_card_<name>_last_played` | Title of the game they last played | `date`, `platform` of the copy played |

A person's sensors keep their entity IDs when they change their name in Home
Assistant.

The counts keep long-term statistics, so history graphs show how the
collection and each backlog change. Sensors update a second after each change
and show as unavailable while Memory Card is stopped.

## Roll a pick from Home Assistant

A pick comes from one person's backlog, so a roll names whose.

With the integration:

- The **Roll a pick** button (`button.memory_card_roll_pick`) rolls for
  whoever presses it.
- The `memory_card.roll_pick` action rolls for whoever calls it. An
  automation names the person with `person: person.ana`. It responds with the
  picked game's `title` and `platforms`.

```yaml
action: memory_card.roll_pick
data:
  person: person.ana
  device: [switch]
  min_local_players: 2
response_variable: pick
```

With MQTT, which cannot tell who sent a message, publish to
`memory_card/pick/roll` with the person's user ID. A message without a person
is ignored, and rolls come once a second at most.

```yaml
action: mqtt.publish
data:
  topic: memory_card/pick/roll
  payload: '{"person": "<user ID>", "device": ["switch"], "min_local_players": 2}'
```

The filters match the Backlog page's picker: `play_status` (a list of play
status keys), `platform`, `device`, `genre`, `game_mode` (lists of slugs) and
`min_local_players`. Each platform, device and store shows its slug in its
[Manage](manage.md) form, and library links use the same slugs.

Rolls need managers set in the options, and the person must be a manager or a
viewer with `viewers_track_progress` on. A person rolls only for themselves.
A roll that names a person, from an automation or MQTT, needs someone Memory
Card knows: a manager, or a person who has already changed something in it,
such as their play status.
