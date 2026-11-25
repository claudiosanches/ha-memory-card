# NFC tags

Put an NFC tag on a game's box, and scanning it with the Home Assistant
companion app starts that copy for you: Memory Card logs a play session for
today in your play and sets your status for the game to playing.

## Link a tag

1. On the [game page](game.md), open the copy's **⋯** menu and choose **Link
   NFC tag**.
2. Within a minute, scan the tag with the Home Assistant app.
3. Memory Card says **Tag linked**.

Any tag Home Assistant reports works. It does not need to be added under
**Settings > Connectivity > Tags** in Home Assistant first. A copy can have
several tags. **Unlink
NFC tags** in the same menu removes them.

A link waits for a scan by the manager who started it, or by a reader with
no user that `tag_readers` does not give to someone else.

## What a scan does

- It logs a session for today and sets your status for the game to playing.
  Scanning the same tag again on the same day adds no other session.
- A game whose status counts as finished goes back to playing too, unless the
  `tag_scan_finished` option is set to `keep`.
- Pages open in Memory Card show the scan within a second.

A scan logs play for managers, and for viewers when `viewers_track_progress`
is on. Nothing is logged while no managers are set.

## Readers without a user

A reader with no Home Assistant user, such as an ESPHome NFC reader, logs
play for the person the `tag_readers` option names for it, and nothing
otherwise.

1. Find the reader's device ID: in Home Assistant, open the reader under
   **Settings > Devices & services > Devices**. The ID is the last part of the
   page's address. A reader no option names also logs its device ID in the
   app's **Log** tab when it scans a linked tag.
2. Find the person's user ID under **Settings > Status > Your access** in Memory Card,
   opened as that person.
3. In the app's **Configuration** tab, add the pair to `tag_readers`, save and
   restart the app:

   ```yaml
   tag_readers:
     - device: 4f1c9e0d8a7b6c5d4e3f2a1b0c9d8e7f
       person: your-user-id
   ```

## Checking it works

Memory Card hears scans through the Home Assistant API, which is why the app
asks for access to it. **Settings > Status > NFC tag scans** says whether it
is listening.

To try a scan without a tag, fire a `tag_scanned` event with
`{"tag_id": "your-tag-id"}` under **Developer tools > Events** in Home
Assistant. The scan counts as the user who fires it.
