#!/bin/sh
# Regenerates the OpenAPI document and the TypeScript types the web app uses.
# With --check, generates into a temp dir and fails if the files in the repo differ.
set -eu
cd "$(dirname "$0")/.."

spec=$PWD/web/openapi.json
types=$PWD/web/src/api/schema.d.ts
if [ "${1:-}" = "--check" ]; then
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    spec=$tmp/openapi.json
    types=$tmp/schema.d.ts
fi

cargo run --quiet --bin memory-card -- openapi > "$spec"
npm exec --silent -w web -- openapi-typescript "$spec" -o "$types" >/dev/null

if [ "${1:-}" = "--check" ]; then
    diff -u web/openapi.json "$spec"
    diff -u web/src/api/schema.d.ts "$types"
    echo "generated API files are up to date"
fi
