# Releasing

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
