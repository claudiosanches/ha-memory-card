# Releasing

Pull requests go into `dev`. `main` holds the latest release: Home Assistant
reads `memory-card/config.yaml` (the version and the image tag),
`memory-card/DOCS.md` and `memory-card/CHANGELOG.md` from `main`, so `main`
moves only once a release's image exists. `main` stays the default branch,
since Home Assistant clones the default branch, so pull requests pick `dev` as
their base.

Each change goes under **Unreleased** in `memory-card/CHANGELOG.md` as it
lands, in an Added, Changed, Deprecated, Removed, Fixed or Security section,
as [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) describes.

1. On an up-to-date `dev`, run `make release`. It picks the version from the
   Unreleased entries: Removed bumps the major version (the minor before
   1.0), Added, Changed or Deprecated the minor, and only Fixed or Security
   the patch. `make release VERSION=X.Y.Z` picks it instead. It sets the
   version in `memory-card/config.yaml`, `Cargo.toml`, `web/package.json` and
   their lock files, moves the entries into a `## [X.Y.Z]` section dated
   today, and commits and tags `vX.Y.Z` on `dev`.
2. Push both: `git push origin dev vX.Y.Z`. The release workflow checks the
   versions, builds both architectures, smoke-tests the image, pushes
   `ghcr.io/claudiosanches/memory-card:X.Y.Z`, creates the GitHub release with
   the changelog's entries, and fast-forwards `main` to the tag.
3. Deploy the docs site from `main` with `make docs-deploy`.

The workflow moves `main` with its own token, so a rule that requires pull
requests on `main` blocks it. Protect `main` against force pushes and deletion
instead.

To undo a release that is not pushed yet, run `git tag -d vX.Y.Z` and
`git reset --hard HEAD~1` on `dev`. A pushed tag stays: if the workflow fails
on a flaky step, rerun it; on a real bug, fix it on `dev` with a changelog
entry and release again.

## The integration

The [Memory Card integration](https://github.com/claudiosanches/ha-memory-card-integration)
keeps its own `CHANGELOG.md` and the same `make release`. HACS installs
releases rather than the default branch, so it has no `dev`: pull requests go
into `main`, `make release` commits and tags there, and
`git push origin main vX.Y.Z` runs its checks and creates the GitHub release,
which HACS offers as an update.
