# Development

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
| `make ha-up` | Local Home Assistant with the app from this tree, see [Local Home Assistant](local-home-assistant.md) |
| `make seed` | Fills an empty collection in `make run` with test data, plus play for `sam`, `alex`, `jordan` and `robin` |
| `make screenshots` | Takes the screenshots into `docs/public/images` from test data. Only the docs site keeps them, out of git, and every page links them by their `https://ha-memory-card.pages.dev/images/` URL. Needs IGDB credentials in `.data/options.json`. `ONLY=themes/` retakes only the images whose names start with it |
| `make docs-dev` | The docs site with live reload, see [Docs site](#docs-site) |
| `make docs-deploy` | Builds the docs site and deploys it to Cloudflare Pages, with the screenshots; refuses to run when a screenshot the docs link is missing |
| `make reset-db` | Deletes the local database and covers, keeping `options.json`. Needed after a migration changes before 0.1 |
| `make release` | Commits and tags the next release on `dev`, see [Releasing](releasing.md) |

The server answers only requests that name a Home Assistant user, so open
the app through `make proxy` or `make web-dev`. Both sign in as `DEV_USER`
(`dev` by default), whom `make run` makes a manager. In the proxy, an
`e2e_user` cookie picks another user; an empty one signs out.
`DEV_USER=sam make web-dev` signs in as one of the people `make seed` adds.

The server reads these variables; `make run` sets the first four,
`MEMORY_CARD_MANAGERS` and `MEMORY_CARD_VIEWERS_TRACK_PROGRESS`.

| Variable | Default | Purpose |
|---|---|---|
| `MEMORY_CARD_DATA_DIR` | `/data` | Database, covers, `options.json` |
| `MEMORY_CARD_TMP_DIR` | `tmp` in the data dir | Temporary files, cleared at startup; outside the data dir or under its `tmp` |
| `MEMORY_CARD_LISTEN` | `0.0.0.0:8099` | Listen address |
| `MEMORY_CARD_TRUSTED_PEER` | `172.30.32.2` | Comma-separated peers allowed to connect |
| `MEMORY_CARD_LOG_LEVEL` | from `options.json` | Overrides `log_level` |
| `MEMORY_CARD_IGDB_CLIENT_ID` | from `options.json` | Overrides `igdb_client_id` |
| `MEMORY_CARD_IGDB_CLIENT_SECRET` | from `options.json` | Overrides `igdb_client_secret` |
| `MEMORY_CARD_MANAGERS` | from `options.json` | Comma-separated user IDs; overrides `managers` |
| `MEMORY_CARD_EXTERNAL_SERVICES` | from `options.json` | `true` or `false`; overrides `external_services` |
| `MEMORY_CARD_VIEWERS_TRACK_PROGRESS` | from `options.json` | `true` or `false`; overrides `viewers_track_progress` |
| `MEMORY_CARD_SENSORS` | from `options.json` | `mqtt`, `integration` or `off`; overrides `sensors` |
| `MEMORY_CARD_INTEGRATION_LISTEN` | `0.0.0.0:8100` | Address of the feed the Memory Card integration reads |
| `MEMORY_CARD_INTEGRATION_PEER` | `172.30.32.1, fd0c:ac1e:2100::1` | Comma-separated peers allowed to read that feed |
| `MEMORY_CARD_TAG_SCAN_FINISHED` | from `options.json` | `playing` or `keep`; overrides `tag_scan_finished` |
| `MEMORY_CARD_TAG_READERS` | from `options.json` | `device=person,...` pairs of Home Assistant device and user IDs; overrides `tag_readers` |
| `MEMORY_CARD_CURRENCY` | from `options.json` | Three-letter code; overrides `currency` |
| `MEMORY_CARD_DATE_FORMAT` | from `options.json` | `language`, `dmy`, `mdy` or `ymd`; overrides `date_format` |
| `MEMORY_CARD_MQTT_HOST` | unset | MQTT broker for the sensors, instead of asking the Supervisor |
| `MEMORY_CARD_MQTT_PORT` | `1883` | Port of that broker |
| `MEMORY_CARD_MQTT_USERNAME` | unset | Broker user |
| `MEMORY_CARD_MQTT_PASSWORD` | unset | Broker password |
| `MEMORY_CARD_SUPERVISOR_URL` | `http://supervisor` | Supervisor API, asked for the broker when `SUPERVISOR_TOKEN` is set |
| `MEMORY_CARD_BARCODE_URL` | UPCitemdb's trial lookup | Barcode lookup endpoint, for tests |
| `MEMORY_CARD_HA_URL`, `MEMORY_CARD_HA_TOKEN` | none | A Home Assistant URL (`http://`) and long-lived token to hear tag scans outside Home Assistant |

The connection to `MEMORY_CARD_MQTT_HOST` has no TLS, so the broker password
and messages cross the network in the clear. Use a broker on the same machine
or a network you trust. TLS support is planned. Inside Home Assistant this does not
apply: the Mosquitto app is reached over Home Assistant's internal network,
and it offers no TLS there.

## Queries

All SQL lives in `server/src/db`. Write queries with the `sqlx::query!`
macros, which check them against the schema when the server builds. Filters
and table or column names picked at runtime go through `db::sql::Sql`, which
takes only `&'static str` text and binds every value. `server/clippy.toml` and
`server/tests/sql_in_db.rs` fail the checks on a query outside `db`, and on
`QueryBuilder` or `AssertSqlSafe` outside `db/sql.rs`. Run `make sqlx` after
adding or changing a query.

## Docs site

`docs/` is also the site at https://ha-memory-card.pages.dev, built with
VitePress from `docs/.vitepress/`. `docs/index.md` is the landing page,
`docs/about.md` the About page, `docs/README.md` the guide's index at `/docs/`,
and the other guide pages sit under `/docs/`. The developer pages (this one, Local Home Assistant and
Releasing) stay out of the site. A new guide page needs a line in
`docs/README.md` and in the sidebar in `docs/.vitepress/config.mjs`.

VitePress and wrangler are locked in `docs/package-lock.json`, apart from the
app's dependencies; `make docs` installs them with `npm ci`, without install
scripts. Before the first `make docs-deploy`, sign in to Cloudflare with
`docs/node_modules/.bin/wrangler login`. A deploy from `main` updates the site; from another
branch it gets a preview URL.
