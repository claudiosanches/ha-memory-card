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

1. On `dev`, up to date with `origin/dev`, run `make release`. It picks the
   version from the Unreleased entries: Removed bumps the major version (the
   minor before 1.0), Added, Changed or Deprecated the minor, and only Fixed
   or Security the patch. `make release VERSION=X.Y.Z` picks it instead. It
   sets the version in `memory-card/config.yaml`, `Cargo.toml`,
   `web/package.json`, `web/openapi.json` and the lock files, moves the
   entries into a `## [X.Y.Z]` section dated today, and commits and tags
   `vX.Y.Z` on `dev`.
2. Push both at once: `git push --atomic origin dev vX.Y.Z`. The release
   workflow checks that the tag is on `dev` and matches the versions, runs
   CI, builds both architectures and smoke-tests each, pushes
   `ghcr.io/claudiosanches/memory-card:X.Y.Z` and creates the GitHub release
   with the changelog's entries.
3. Once it succeeds, move `main` to the release:
   `git push origin 'vX.Y.Z^{commit}:refs/heads/main'`. Home Assistant offers
   the update from then on. The workflow cannot do this itself: its token may
   not move a branch onto workflow changes.
4. Deploy the docs site from `main`: `git switch main`, `git pull --ff-only`,
   then `make docs-deploy`.

Pushing workflow changes needs the `workflow` scope on your GitHub token:
`gh auth refresh --hostname github.com --scopes workflow` adds it.

If a job fails on a flaky step, rerun it. A version already on ghcr.io and a
release that exists are kept as they are. The tag cannot be moved or deleted,
so a real bug takes a fix on `dev` with a changelog entry and the next
release.

To undo a release that is not pushed yet, run `git tag -d vX.Y.Z` and
`git reset --hard HEAD~1` on `dev`.

## The integration

The [Memory Card integration](https://github.com/claudiosanches/ha-memory-card-integration)
keeps its own `CHANGELOG.md` and the same `make release`. HACS installs
releases rather than the default branch, so it has no `dev`: pull requests go
into `main`, `make release` commits and tags there, and
`git push --atomic origin main vX.Y.Z` runs its checks and creates the GitHub
release, which HACS offers as an update.
