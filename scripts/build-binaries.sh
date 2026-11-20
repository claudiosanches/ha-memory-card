#!/bin/sh
# Cross-compiles static musl release binaries into dist/<docker arch>/memory-card.
# Needs zig and cargo-zigbuild on PATH and the web app built first.
set -eu
cd "$(dirname "$0")/.."

if [ ! -f web/dist/index.html ]; then
    echo "web/dist is missing: run 'npm run build' first" >&2
    exit 1
fi

for pair in x86_64-unknown-linux-musl:amd64 aarch64-unknown-linux-musl:arm64; do
    target=${pair%%:*}
    arch=${pair##*:}
    cargo zigbuild --locked --release --target "$target" --bin memory-card
    mkdir -p "dist/$arch"
    cp "target/$target/release/memory-card" "dist/$arch/memory-card"
done
