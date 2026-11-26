#!/bin/sh
# Loads each architecture's image from a multi-platform OCI archive, the one
# a release publishes, and runs the smoke test on it; arm64 runs emulated.
# skopeo writes a docker save tarball, since its own daemon client is too old
# for current Docker.
# Usage: scripts/smoke-archive.sh <oci-archive>
set -eu

archive=$1
scripts=$(dirname "$0")
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"; docker rmi memory-card:smoke-amd64 memory-card:smoke-arm64 >/dev/null 2>&1 || true' EXIT

for arch in amd64 arm64; do
    image=memory-card:smoke-$arch
    skopeo --override-os linux --override-arch "$arch" copy --quiet \
        "oci-archive:$archive" "docker-archive:$tmp/$arch.tar:$image"
    docker load --quiet --input "$tmp/$arch.tar" >/dev/null
    DOCKER_DEFAULT_PLATFORM=linux/$arch "$scripts/smoke-image.sh" "$image"
    docker rmi "$image" >/dev/null
done
