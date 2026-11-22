#!/bin/sh
# A throwaway Home Assistant with a real Supervisor, run in Home Assistant's
# app devcontainer, with this app built from the working tree.
# Usage: scripts/ha-demo.sh up|stop|destroy|logs
set -eu
cd "$(dirname "$0")/.."

name=${HA_NAME:-memory-card-ha}
port=${HA_PORT:-7123}
image=${HA_IMAGE:-ghcr.io/home-assistant/devcontainer:6-apps}
label=memory-card.ha-demo
volumes="docker containerd supervisor"
state=target/ha
apps=$state/local

ours() {
    [ "$(docker container inspect -f "{{index .Config.Labels \"$label\"}}" "$name" 2>/dev/null)" = 1 ]
}

exists() {
    docker container inspect "$name" >/dev/null 2>&1
}

running() {
    [ "$(docker container inspect -f '{{.State.Running}}' "$@" 2>/dev/null)" = true ]
}

refuse_foreign() {
    if exists && ! ours; then
        echo "A container named $name exists that this script did not create. Set HA_NAME." >&2
        exit 1
    fi
}

# The Supervisor builds the app from this folder. Its Dockerfile expects the
# repository root as the build context, so the binary and rootfs sit at the
# same paths here. Without `image` the Supervisor builds instead of pulling,
# and a new version each time lets an update apply config.yaml changes.
stage() {
    arch=$1
    stamp=$(date +%Y%m%d%H%M%S)
    app=$apps/memory-card
    mkdir -p "$apps"
    rm -rf "$app"
    mkdir -p "$app/dist/$arch" "$app/memory-card"
    for file in memory-card/*; do
        case $file in
        memory-card/rootfs) cp -R "$file" "$app/memory-card/" ;;
        memory-card/config.yaml)
            sed -e '/^image:/d' -e "s/^version: \"\(.*\)\"$/version: \"\1-dev.$stamp\"/" \
                "$file" >"$app/config.yaml"
            ;;
        *) cp -R "$file" "$app/" ;;
        esac
    done
    cp "dist/$arch/memory-card" "$app/dist/$arch/"
}

up() {
    arch=$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/')
    refuse_foreign
    stage "$arch"

    if ! exists; then
        for volume in $volumes; do
            docker volume create --label "$label=1" "$name-$volume" >/dev/null
        done
        # Privileged for systemd and Docker inside, as the devcontainer needs.
        # Home Assistant serves on port 80 and redirects 8123 there.
        docker run -d --privileged --name "$name" --label "$label=1" \
            -p "127.0.0.1:$port:80" \
            -v "$name-docker:/var/lib/docker" \
            -v "$name-containerd:/var/lib/containerd" \
            -v "$name-supervisor:/mnt/supervisor" \
            -v "$PWD/$apps:/mnt/supervisor/apps/local" \
            --tmpfs /tmp \
            -e SUPERVISOR_CHANNEL=stable \
            "$image" >/dev/null
    elif ! running "$name"; then
        docker start "$name" >/dev/null
    fi

    if ! docker exec "$name" test -f /mnt/supervisor/apps/local/memory-card/config.yaml; then
        echo "The container does not see $apps. Run 'make ha-stop' and try again." >&2
        exit 1
    fi

    tries=0
    until docker exec "$name" docker info >/dev/null 2>&1; do
        tries=$((tries + 1))
        if [ "$tries" -gt 120 ]; then
            echo "Docker did not start in the container" >&2
            exit 1
        fi
        sleep 1
    done

    if [ "$(docker exec "$name" docker container inspect -f '{{.State.Running}}' hassio_supervisor 2>/dev/null)" != true ]; then
        docker exec -d "$name" bash -c 'supervisor_run >/var/log/supervisor_run.log 2>&1'
    fi

    node scripts/ha-demo.mjs "http://127.0.0.1:$port" "$state/demo.json"
}

case ${1:-} in
up)
    up
    ;;
stop)
    refuse_foreign
    if running "$name"; then
        docker stop -t 120 "$name" >/dev/null
    fi
    ;;
destroy)
    refuse_foreign
    docker rm -f "$name" >/dev/null 2>&1 || true
    for volume in $volumes; do
        if [ "$(docker volume inspect -f "{{index .Labels \"$label\"}}" "$name-$volume" 2>/dev/null)" = 1 ]; then
            docker volume rm "$name-$volume" >/dev/null
        fi
    done
    rm -rf "$state"
    ;;
logs)
    refuse_foreign
    docker exec "$name" docker logs -f app_local_memory_card
    ;;
*)
    echo "usage: $0 up|stop|destroy|logs" >&2
    exit 1
    ;;
esac
