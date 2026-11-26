# Releasing

Each change goes under **Unreleased** in `memory-card/CHANGELOG.md` as it
lands, in an Added, Changed, Deprecated, Removed, Fixed or Security section,
as [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) describes.

Home Assistant reads `version` from `memory-card/config.yaml` on `main` and
pulls the image with that tag, so the image must exist before `main` changes.

1. On an up-to-date `main`, run `make release`. It picks the version from the
   Unreleased entries: Removed bumps the major version (the minor before
   1.0), Added, Changed or Deprecated the minor, and only Fixed or Security
   the patch. `make release VERSION=X.Y.Z` picks it instead. It sets the
   version in `memory-card/config.yaml`, `Cargo.toml`, `web/package.json` and
   their lock files, moves the entries into a `## [X.Y.Z]` section dated
   today, and commits and tags `vX.Y.Z` on a new `release/X.Y.Z` branch.
2. Push both: `git push origin release/X.Y.Z vX.Y.Z`. The release workflow
   checks the versions, builds both architectures, smoke-tests the image,
   pushes `ghcr.io/claudiosanches/memory-card:X.Y.Z` and creates the GitHub
   release with the changelog's entries.
3. Once it succeeds, merge the branch into `main` with a merge commit or
   `git merge --ff-only`. A squash or rebase merge leaves the tagged commit
   off `main`.

To undo a release that is not pushed yet, delete the tag and the branch:
`git tag -d vX.Y.Z` and, from `main`, `git branch -D release/X.Y.Z`.

## The integration

The [Memory Card integration](https://github.com/claudiosanches/ha-memory-card-integration)
releases from its own repository the same way, keeping its own
`CHANGELOG.md`. HACS reads releases rather than `main`, so there is no
release branch: `make release` commits and tags on `main`, and
`git push origin main vX.Y.Z` runs its checks and creates the GitHub release,
which HACS offers as an update.
