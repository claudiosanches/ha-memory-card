#!/bin/sh
# Prepares a release on dev: picks the version from the changelog, sets it in
# every version field, dates the changelog section, then commits and tags.
# Pushing is left to you; the release workflow moves main once the image is out.
#
# The Unreleased entries pick the version: Removed bumps the major version
# (the minor before 1.0), Added, Changed or Deprecated the minor, and only
# Fixed or Security the patch. The first release is 0.1.0.
# Usage: scripts/release.sh [X.Y.Z]
set -eu
cd "$(dirname "$0")/.."

changelog=memory-card/CHANGELOG.md
repo_url=https://github.com/claudiosanches/ha-memory-card

fail() {
    echo "$*" >&2
    exit 1
}

[ -z "$(git status --porcelain)" ] || fail "Commit or stash your changes first."
[ "$(git branch --show-current)" = dev ] || fail "Run it on dev."

unreleased=$(awk '/^## \[Unreleased\]/ { found = 1; next } found && /^## / { exit } found { print }' "$changelog")
printf '%s\n' "$unreleased" | grep -q '^- ' || fail "$changelog has nothing under Unreleased."
types=$(printf '%s\n' "$unreleased" | sed -n 's/^### //p')
for type in $types; do
    case $type in
    Added | Changed | Deprecated | Removed | Fixed | Security) ;;
    *) fail "$changelog has an unknown section under Unreleased: $type" ;;
    esac
done

previous=$(sed -n 's/^## \[\([0-9][0-9.]*\)\] - .*/\1/p' "$changelog" | head -n 1)

if [ -n "${1:-}" ]; then
    version=$1
    printf '%s\n' "$version" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || fail "VERSION takes X.Y.Z."
    if [ -n "$previous" ]; then
        newest=$(printf '%s\n%s\n' "$previous" "$version" | sort -V | tail -n 1)
        if [ "$version" = "$previous" ] || [ "$newest" != "$version" ]; then
            fail "$version is not after $previous."
        fi
    fi
elif [ -z "$previous" ]; then
    version=0.1.0
else
    case $types in
    *Removed*) bump="major" ;;
    *Added* | *Changed* | *Deprecated*) bump="minor" ;;
    *) bump="patch" ;;
    esac
    IFS=. read -r major minor patch <<EOF
$previous
EOF
    if [ "$bump" = major ] && [ "$major" = 0 ]; then
        bump="minor"
    fi
    case $bump in
    major) version=$((major + 1)).0.0 ;;
    minor) version=$major.$((minor + 1)).0 ;;
    patch) version=$major.$minor.$((patch + 1)) ;;
    esac
fi

! git rev-parse -q --verify "refs/tags/v$version" >/dev/null || fail "The tag v$version exists."

if [ -n "$previous" ]; then
    version_url="$repo_url/compare/v$previous...v$version"
else
    version_url="$repo_url/releases/tag/v$version"
fi
awk -v version="$version" -v date="$(date +%F)" -v repo="$repo_url" -v version_url="$version_url" '
    /^## \[Unreleased\]/ { print; print ""; print "## [" version "] - " date; next }
    /^\[Unreleased\]: / {
        print "[Unreleased]: " repo "/compare/v" version "...dev"
        print "[" version "]: " version_url
        linked = 1
        next
    }
    { print }
    END { if (!linked) { print ""; print "[Unreleased]: " repo "/compare/v" version "...dev"; print "[" version "]: " version_url } }
' "$changelog" >"$changelog.new"
mv "$changelog.new" "$changelog"

sed -i "s/^version: \".*\"\$/version: \"$version\"/" memory-card/config.yaml
sed -i "/^\[workspace.package\]/,/^\[/ s/^version = \".*\"\$/version = \"$version\"/" Cargo.toml
cargo update --workspace --offline --quiet
node -e '
const fs = require("fs");
const version = process.argv[1];
const edits = [
    ["web/package.json", (json) => { json.version = version; }],
    ["package-lock.json", (json) => { json.packages.web.version = version; }],
];
for (const [file, edit] of edits) {
    const json = JSON.parse(fs.readFileSync(file, "utf8"));
    edit(json);
    fs.writeFileSync(file, JSON.stringify(json, null, 2) + "\n");
}
' "$version"
./scripts/check-version.sh "$version"

git commit -q -am "chore: Release $version"
git tag -a "v$version" -m "Memory Card $version"

cat <<EOF
Tagged v$version. To publish it:

    git push origin dev v$version

The release workflow builds and pushes the image, creates the GitHub release,
then moves main to v$version.
EOF
