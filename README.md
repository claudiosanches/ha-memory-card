# Memory Card

A video game collection manager that runs as a Home Assistant app. It tracks
the games you own across consoles, the platform and region releases of each
game, and every copy you have, physical or digital.

## Install

1. In Home Assistant, go to **Settings > Apps > App store**.
2. Open the menu, choose **Repositories** and add
   `https://github.com/claudiosanches/ha-memory-card`.
3. Install **Memory Card** and start it. It appears in the sidebar.

See [DOCS.md](memory-card/DOCS.md) for configuration.

## Development

Requirements: the Rust toolchain from `rust-toolchain.toml`, Node.js 24.
Changing SQL queries needs sqlx-cli
(`cargo install sqlx-cli --version 0.9.1 --locked --no-default-features --features sqlite`).
Building the image and `make ha-up` also need zig 0.15.2, cargo-zigbuild and
Docker. `make ha-up` also needs an x86-64 Linux machine with KVM.

`make` lists the targets. The common ones:

| Target | What it does |
|---|---|
| `make run` | Server on http://127.0.0.1:8099/ with data in `.data/` |
| `make proxy` | Serves that server at http://127.0.0.1:18100/api/hassio_ingress/e2e-token/, like Home Assistant ingress |
| `make web-dev` | Vite with live reload, `/api` proxied to `make run` |
| `make test` | Rust and web unit tests |
| `make e2e` | Playwright tests behind the ingress stand-in |
| `make check` | Everything CI runs except the image |
| `make smoke` | Builds the amd64 image and runs it like Home Assistant |
| `make api` | Regenerates the OpenAPI document and TypeScript types after an API change |
| `make sqlx` | Regenerates `server/.sqlx` after adding or changing a query |
| `make ha-up` | Local Home Assistant with the app from this tree, see below |
| `make seed` | Fills an empty collection in `make run` with test data |
| `make reset-db` | Deletes the local database and covers, keeping `options.json`. Needed after a migration changes before 0.1 |

The server answers only requests that name a Home Assistant user, so open
the app through `make proxy` or `make web-dev`. Both sign in as `DEV_USER`
(`dev` by default), whom `make run` makes a manager. In the proxy, an
`e2e_user` cookie picks another user; an empty one signs out.

The server reads these variables; `make run` sets the first four and
`MEMORY_CARD_MANAGERS`.

| Variable | Default | Purpose |
|---|---|---|
| `MEMORY_CARD_DATA_DIR` | `/data` | Database, covers, `options.json` |
| `MEMORY_CARD_TMP_DIR` | `tmp` in the data dir | Temporary files, cleared at startup |
| `MEMORY_CARD_LISTEN` | `0.0.0.0:8099` | Listen address |
| `MEMORY_CARD_TRUSTED_PEER` | `172.30.32.2` | Comma-separated peers allowed to connect |
| `MEMORY_CARD_LOG_LEVEL` | from `options.json` | Overrides `log_level` |
| `MEMORY_CARD_IGDB_CLIENT_ID` | from `options.json` | Overrides `igdb_client_id` |
| `MEMORY_CARD_IGDB_CLIENT_SECRET` | from `options.json` | Overrides `igdb_client_secret` |
| `MEMORY_CARD_MANAGERS` | from `options.json` | Comma-separated user IDs; overrides `managers` |
| `MEMORY_CARD_EXTERNAL_SERVICES` | from `options.json` | `true` or `false`; overrides `external_services` |
| `MEMORY_CARD_VIEWERS_TRACK_PROGRESS` | from `options.json` | `true` or `false`; overrides `viewers_track_progress` |
| `MEMORY_CARD_SENSORS` | from `options.json` | `mqtt` or `off`; overrides `sensors` |
| `MEMORY_CARD_TAG_SCAN_FINISHED` | from `options.json` | `playing` or `keep`; overrides `tag_scan_finished` |
| `MEMORY_CARD_CURRENCY` | from `options.json` | Three-letter code; overrides `currency` |
| `MEMORY_CARD_MQTT_HOST` | unset | MQTT broker for the sensors, instead of asking the Supervisor |
| `MEMORY_CARD_MQTT_PORT` | `1883` | Port of that broker |
| `MEMORY_CARD_MQTT_USERNAME` | unset | Broker user |
| `MEMORY_CARD_MQTT_PASSWORD` | unset | Broker password |
| `MEMORY_CARD_SUPERVISOR_URL` | `http://supervisor` | Supervisor API, asked for the broker when `SUPERVISOR_TOKEN` is set |
| `MEMORY_CARD_BARCODE_URL` | UPCitemdb's trial lookup | Barcode lookup endpoint, for tests |
| `MEMORY_CARD_HA_URL`, `MEMORY_CARD_HA_TOKEN` | none | A Home Assistant URL (`http://`) and long-lived token to hear tag scans outside Home Assistant |

The connection to `MEMORY_CARD_MQTT_HOST` has no TLS, so the broker password
and messages cross the network in the clear. Use a broker on the same machine
or a network you trust. TLS is planned once the MQTT library,
[rumqttc](https://github.com/bytebeamio/rumqtt), supports it without crates
that have open security advisories. Inside Home Assistant this does not
apply: the Mosquitto app is reached over Home Assistant's internal network,
and it offers no TLS there.

### Local Home Assistant

`make ha-up` runs [Home Assistant OS](https://github.com/home-assistant/operating-system)
in a virtual machine and installs the app built from this tree. The VM runs
in QEMU inside an unprivileged Docker container that gets only `/dev/kvm`,
so Home Assistant has its own kernel and changes nothing on your machine. It
uses 2 CPUs and 4 GB of RAM. The first run downloads Home Assistant OS (about
500 MB, kept in `target/haos-image`) and then Home Assistant, and takes a few
minutes. It also:

- finishes onboarding with an `owner` administrator, made a Memory Card
  manager, and adds a `viewer` regular user. Their passwords are printed and
  kept in `target/haos/demo.json`.
- installs Mosquitto and the MQTT integration, for the sensors.
- copies the IGDB credentials from `.data/options.json`, if set, again on
  every run when they change.
- the first time, fills the collection with test data (`scripts/seed.mjs`):
  about 40 games on 13 platforms with devices, stores, prices in four
  currencies, play sessions, custom play statuses, a bundle and a wishlist,
  linked on IGDB when the credentials are set. A few titles have no exact
  match there, which leaves something for **Review close matches**.
- adds a **Games** dashboard with the app's sensors.

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

## Releasing

Home Assistant reads `version` from `memory-card/config.yaml` on `main` and
pulls the image with that tag, so the image must exist before `main` changes.

1. On a `release/X.Y.Z` branch, set `X.Y.Z` in `memory-card/config.yaml`,
   `Cargo.toml` and `web/package.json`, run `cargo build` and `npm install` to
   update the lock files, and add a `## X.Y.Z` section to
   `memory-card/CHANGELOG.md`.
2. Publish a GitHub release tagged `vX.Y.Z` from that branch. The release
   workflow checks the versions, builds both architectures, smoke-tests the
   image and pushes `ghcr.io/claudiosanches/memory-card:X.Y.Z`.
3. Merge the branch into `main`.

## Credits

- Interface icons from [Tabler Icons](https://tabler.io/icons) (MIT).
- Game metadata and covers from [IGDB](https://www.igdb.com), when IGDB credentials are set.
- Product names for scanned barcodes from [UPCitemdb](https://www.upcitemdb.com).

## License

Memory Card is licensed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).
