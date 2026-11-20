#!/bin/sh
# Fails unless every version field in the repo equals the given version.
# Usage: scripts/check-version.sh 0.1.0
set -eu
cd "$(dirname "$0")/.."

expected=$1
status=0

check() {
    if [ "$2" != "$expected" ]; then
        echo "$1 has version '$2', expected '$expected'" >&2
        status=1
    fi
}

check memory-card/config.yaml "$(sed -n 's/^version: "\(.*\)"$/\1/p' memory-card/config.yaml)"
check Cargo.toml "$(sed -n '/^\[workspace.package\]/,/^\[/s/^version = "\(.*\)"$/\1/p' Cargo.toml)"
check web/package.json "$(node -p 'require("./web/package.json").version')"
grep -q "^## $expected\$" memory-card/CHANGELOG.md || {
    echo "memory-card/CHANGELOG.md has no '## $expected' section" >&2
    status=1
}

exit "$status"
