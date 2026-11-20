#!/bin/sh
# Regenerates the OpenAPI document and the TypeScript types the web app uses.
set -eu
cd "$(dirname "$0")/.."
cargo run --quiet --bin memory-card -- openapi > web/openapi.json
npm run --silent gen:api -w web
