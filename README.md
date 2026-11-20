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

```sh
npm install
npm run build                 # web app into web/dist
cargo run -- serve            # needs the env below outside Home Assistant
```

Local run outside Home Assistant:

```sh
MEMORY_CARD_DATA_DIR=.data \
MEMORY_CARD_TMP_DIR=.data/tmp \
MEMORY_CARD_LISTEN=127.0.0.1:8099 \
MEMORY_CARD_TRUSTED_PEER=127.0.0.1 \
cargo run -- serve
```

`npm run dev -w web` starts Vite with `/api` proxied to that server.

| Variable | Default | Purpose |
|---|---|---|
| `MEMORY_CARD_DATA_DIR` | `/data` | Database, covers, `options.json` |
| `MEMORY_CARD_TMP_DIR` | `/tmp/memory-card` | Temporary files, cleared at startup |
| `MEMORY_CARD_LISTEN` | `0.0.0.0:8099` | Listen address |
| `MEMORY_CARD_TRUSTED_PEER` | `172.30.32.2` | Comma-separated peers allowed to connect |
| `MEMORY_CARD_LOG_LEVEL` | from `options.json` | Overrides `log_level` |
| `MEMORY_CARD_IGDB_CLIENT_ID` | from `options.json` | Overrides `igdb_client_id` |
| `MEMORY_CARD_IGDB_CLIENT_SECRET` | from `options.json` | Overrides `igdb_client_secret` |

Checks, as CI runs them:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo deny check
npm run lint && npm run typecheck && npm test
npm run licenses
./scripts/gen-api.sh && git diff --exit-code web/openapi.json web/src/api/schema.d.ts
npm run build && cargo build && npx playwright install chromium && npm run e2e
```

After changing an API handler, run `./scripts/gen-api.sh` to regenerate
`web/openapi.json` and `web/src/api/schema.d.ts`.

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

GPL-3.0-only. See [LICENSE](LICENSE).
