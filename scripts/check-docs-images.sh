#!/bin/sh
# Fails unless every screenshot that the docs link on the docs site is in the
# built site, so a deploy cannot remove one.
# Usage: scripts/check-docs-images.sh, after make docs
set -eu
cd "$(dirname "$0")/.."

base=https://ha-memory-card.pages.dev/images/
site=docs/.vitepress/dist/images

urls=$(grep -rhoE 'https://ha-memory-card\.pages\.dev/images/[A-Za-z0-9/._-]+' \
    docs/*.md docs/pt-br/*.md docs/es/*.md docs/.vitepress/theme | sort -u)
[ -n "$urls" ] || {
    echo "Found no screenshot links" >&2
    exit 1
}

missing=0
for url in $urls; do
    if [ ! -s "$site/${url#"$base"}" ]; then
        echo "Not in the built site: ${url#"$base"}" >&2
        missing=1
    fi
done
if [ "$missing" -ne 0 ]; then
    echo "Run make screenshots first; deploying now would remove these from the site." >&2
    exit 1
fi
