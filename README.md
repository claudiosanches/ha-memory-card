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
Building the image also needs zig 0.15.2, cargo-zigbuild and Docker.

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
| `make reset-db` | Deletes the local database and covers, keeping `options.json`. Needed after a migration changes before 0.1 |

The server reads these variables; `make run` sets the first four.

| Variable | Default | Purpose |
|---|---|---|
| `MEMORY_CARD_DATA_DIR` | `/data` | Database, covers, `options.json` |
| `MEMORY_CARD_TMP_DIR` | `/tmp/memory-card` | Temporary files, cleared at startup |
| `MEMORY_CARD_LISTEN` | `0.0.0.0:8099` | Listen address |
| `MEMORY_CARD_TRUSTED_PEER` | `172.30.32.2` | Comma-separated peers allowed to connect |
| `MEMORY_CARD_LOG_LEVEL` | from `options.json` | Overrides `log_level` |
| `MEMORY_CARD_IGDB_CLIENT_ID` | from `options.json` | Overrides `igdb_client_id` |
| `MEMORY_CARD_IGDB_CLIENT_SECRET` | from `options.json` | Overrides `igdb_client_secret` |

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

## License

Memory Card is licensed under the GNU General Public License, version 3 or (at your option) any later version. See [LICENSE](LICENSE).
