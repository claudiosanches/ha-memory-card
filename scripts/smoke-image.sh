#!/bin/sh
# Runs a built image the way the Supervisor does (s6 init, /data volume,
# options.json) and checks startup, the UI, the API and a clean stop.
# Usage: scripts/smoke-image.sh <image>
set -eu

image=$1
name=memory-card-smoke-$$
volume=memory-card-smoke-$$

cleanup() {
    docker rm -f "$name" >/dev/null 2>&1 || true
    docker volume rm "$volume" >/dev/null 2>&1 || true
}
trap cleanup EXIT

fail() {
    echo "smoke test failed: $*" >&2
    docker logs "$name" >&2 || true
    exit 1
}

docker volume create "$volume" >/dev/null
docker run --rm -v "$volume:/data" --entrypoint sh "$image" \
    -c 'echo "{\"igdb_client_id\":\"\",\"igdb_client_secret\":\"\",\"log_level\":\"info\"}" > /data/options.json'

# Requests from the host arrive from the bridge gateway, which stands in for
# the Supervisor here.
gateway=$(docker network inspect bridge --format '{{(index .IPAM.Config 0).Gateway}}')
docker run -d --name "$name" -v "$volume:/data" -p 127.0.0.1::8099 \
    -e MEMORY_CARD_TRUSTED_PEER="$gateway" "$image" >/dev/null
port=$(docker port "$name" 8099/tcp | head -n1 | sed 's/.*://')
base="http://127.0.0.1:$port"

for _ in $(seq 1 30); do
    if curl -fsS "$base/api/status" >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

curl -fsS "$base/api/status" | grep -q '"igdb_configured":false' || fail "status endpoint"
curl -fsS -H 'X-Ingress-Path: /api/hassio_ingress/smoke' "$base/" \
    | grep -q '<base href="/api/hassio_ingress/smoke/">' || fail "index page"

started=$(date +%s)
docker stop -t 20 "$name" >/dev/null
elapsed=$(($(date +%s) - started))
[ "$elapsed" -lt 10 ] || fail "stop took ${elapsed}s"
docker logs "$name" 2>&1 | grep -q 'database checkpointed' || fail "no checkpoint on stop"

docker run --rm -v "$volume:/data" --entrypoint sh "$image" -c '
    test -s /data/db/memory-card.sqlite || exit 1
    wal=/data/db/memory-card.sqlite-wal
    test ! -s "$wal"
' || fail "database files after stop"

echo "smoke test passed (stopped in ${elapsed}s)"
