# Local Home Assistant

`make ha-up` runs [Home Assistant OS](https://github.com/home-assistant/operating-system)
in a virtual machine and installs the app built from this tree. The VM runs
in QEMU inside an unprivileged Docker container that gets only `/dev/kvm`,
so Home Assistant has its own kernel and changes nothing on your machine. It
uses 2 CPUs and 4 GB of RAM. The first run downloads Home Assistant OS (about
500 MB, kept in `target/haos-image`) and then Home Assistant, and takes a few
minutes. It also:

- finishes onboarding with an `owner` administrator, made a Memory Card
  manager, and adds a `viewer` regular user, who keeps their own play
  (`viewers_track_progress` is on). Their passwords are printed and kept in
  `target/haos/demo.json`.
- installs Mosquitto and the MQTT integration, for the sensors.
- copies the Memory Card integration from `../ha-memory-card-integration`
  (`HA_INTEGRATION` names another folder), when it is there, and restarts
  Home Assistant when it changed. `HA_SENSORS=integration` switches the
  app's `sensors` option and adds the integration once it is discovered;
  `HA_SENSORS=mqtt` switches back.
- copies the IGDB credentials from `.data/options.json`, if set, again on
  every run when they change.
- the first time, fills the collection with test data (`scripts/seed.mjs`):
  about 40 games on 13 platforms with devices, stores, prices in four
  currencies, play sessions, custom play statuses, a bundle and a wishlist,
  linked on IGDB when the credentials are set. A few titles have no exact
  match there, which leaves something for **Review close matches**. The
  viewer logs some play of their own, for the **Household** page.
- adds a **Games** dashboard with the household's sensors and the owner's.

Open http://127.0.0.1:7123 (`HA_PORT` changes the port). Run `make ha-up`
again after a change to update the app; its data stays. `make ha-logs`
follows the app's log, `make ha-stop` shuts the VM down and `make ha-destroy`
deletes it and its data.

`HA_HTTPS=1 make ha-up` serves Home Assistant over HTTPS, with a certificate
from a test certificate authority made for this VM and saved as
`target/haos/ca.pem`. The next `make ha-up` without it goes back to HTTP.
`HA_LAN=1` opens the port to your network instead of this machine only, so a
phone can reach it; anyone on the network can then reach it too. On a phone
over HTTP, the barcode scanner takes a photo. Over HTTPS it uses the live
camera once the phone trusts the certificate: install `ca.pem` on it as a CA
certificate, and remove it when you are done testing.

`HA_DUCKDNS=<name> make ha-up` serves it at `https://<name>.duckdns.org:7123`
with a Let's Encrypt certificate that phones and the Companion app trust as
they are. It implies HTTPS and network access, and points the name at this
machine's address on your network. Create the name on duckdns.org, then save
its token where the script reads it (in zsh; bash takes
`read -rsp 'DuckDNS token: ' token`):

```sh
(umask 077 && read -rs "token?DuckDNS token: " && print -r -- "$token" > .data/duckdns-token)
```

The certificate is kept in `target/haos-cert`, so rebuilding the VM does not
ask Let's Encrypt for another one, and it renews when it has less than 30 days
left. Your router has to let a public name resolve to a private address.

To scan an NFC tag without one, fire a `tag_scanned` event with
`{"tag_id": "any-id"}` under **Developer tools > Events**. The scan counts as
the user who fires it.

For a root shell on the Home Assistant OS host, run
`docker exec -it memory-card-haos ssh -p 22222 root@127.0.0.1`.
