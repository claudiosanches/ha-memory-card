#!/bin/sh
# Cross-compiles static musl release binaries into dist/<docker arch>/memory-card.
# Needs zig and cargo-zigbuild on PATH and the web app built first.
# Usage: scripts/build-binaries.sh [amd64|arm64 ...], both by default.
set -eu
cd "$(dirname "$0")/.."

if [ ! -f web/dist/index.html ]; then
    echo "web/dist is missing: run 'npm run build' first" >&2
    exit 1
fi

[ $# -gt 0 ] || set -- amd64 arm64
for arch in "$@"; do
    case $arch in
    amd64) target=x86_64-unknown-linux-musl ;;
    arm64) target=aarch64-unknown-linux-musl ;;
    *)
        echo "unknown arch: $arch" >&2
        exit 1
        ;;
    esac
    cargo zigbuild --locked --release --target "$target" --bin memory-card
    mkdir -p "dist/$arch"
    cp "target/$target/release/memory-card" "dist/$arch/memory-card"
done
