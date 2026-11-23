#!/bin/sh
# A throwaway Home Assistant OS in a QEMU virtual machine, with this app
# built from the working tree. QEMU runs in an unprivileged container that
# gets /dev/kvm and nothing else.
# Usage: scripts/ha-demo.sh up|stop|destroy|logs
# HA_HTTPS=1 serves HTTPS with a certificate from a test CA. HA_LAN=1 opens
# the port to the network instead of this machine only. HA_DUCKDNS=<name>
# does both as <name>.duckdns.org, with a Let's Encrypt certificate, the
# DuckDNS token read from .data/duckdns-token. HA_SENSORS=mqtt|integration|off
# sets the app's sensors option. The Memory Card integration is copied from
# HA_INTEGRATION (default ../ha-memory-card-integration) when it exists.
set -eu
cd "$(dirname "$0")/.."

name=${HA_NAME:-memory-card-haos}
port=${HA_PORT:-7123}
label=memory-card.ha-demo
image=memory-card-haos:local
os_version=18.4
os_sha256=756ec485b1b153099ca53e88a5eebd63a801d56b791afe5c6d63ef94a35dc02d
os_file=target/haos-image/haos_ova-$os_version.qcow2.xz
state=target/haos
apps=$state/local
# Outside the VM's state, so a rebuilt VM reuses the certificate.
certs=target/haos-cert
token_file=.data/duckdns-token
integration=${HA_INTEGRATION:-../ha-memory-card-integration}
case ${HA_SENSORS:-} in
'' | mqtt | integration | off) ;;
*)
    echo "HA_SENSORS takes mqtt, integration or off" >&2
    exit 1
    ;;
esac
duckdns=
if [ -n "${HA_DUCKDNS:-}" ]; then
    case $HA_DUCKDNS in
    *[!a-z0-9-]* | *.*)
        echo "HA_DUCKDNS takes the name before .duckdns.org" >&2
        exit 1
        ;;
    esac
    duckdns=$HA_DUCKDNS.duckdns.org
    HA_HTTPS=1
    HA_LAN=1
fi

labelled() {
    [ "$(docker "$1" inspect -f "{{index .${2}Labels \"$label\"}}" "$3" 2>/dev/null)" = 1 ]
}

exists() {
    docker container inspect "$name" >/dev/null 2>&1
}

running() {
    [ "$(docker container inspect -f '{{.State.Running}}' "$name" 2>/dev/null)" = true ]
}

refuse_foreign() {
    if exists && ! labelled container Config. "$name"; then
        echo "A container named $name exists that this script did not create. Set HA_NAME." >&2
        exit 1
    fi
}

ssh_options='-p 22222 -o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new -o LogLevel=ERROR'

vm_ssh() {
    # shellcheck disable=SC2086
    docker exec -i "$name" ssh $ssh_options root@127.0.0.1 "$@"
}

lan_ip() {
    ip -4 route get 1.1.1.1 2>/dev/null | sed -n 's/.* src \([0-9.]*\).*/\1/p'
}

# Issues the certificate from a CA kept in the volume, for localhost and,
# with HA_LAN, this machine's address, and puts it in Home Assistant's ssl
# folder. Prints whether it changed.
issue_certificate() {
    san="DNS:localhost,IP:127.0.0.1${HA_LAN:+,IP:$(lan_ip)}"
    docker exec -e SAN="$san" "$name" sh -euc '
        mkdir -p tls && cd tls
        if [ ! -f ca.pem ]; then
            openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -days 825 \
                -subj "/CN=Memory Card test CA" -keyout ca.key -out ca.pem 2>/dev/null
        fi
        if [ "$(cat san 2>/dev/null)" = "$SAN" ]; then echo same; exit; fi
        openssl req -newkey ec -pkeyopt ec_paramgen_curve:P-256 -nodes -subj "/CN=Memory Card test" \
            -keyout key.pem -out request.pem 2>/dev/null
        printf "subjectAltName=%s\nextendedKeyUsage=serverAuth\n" "$SAN" >extensions
        openssl x509 -req -in request.pem -CA ca.pem -CAkey ca.key -CAcreateserial -days 397 \
            -extfile extensions -out cert.pem 2>/dev/null
        echo "$SAN" >san
        echo changed'
    # shellcheck disable=SC2016
    docker exec "$name" sh -c 'cat tls/cert.pem tls/ca.pem | ssh '"$ssh_options"' root@127.0.0.1 "cat >/mnt/data/supervisor/ssl/memory-card-demo.pem"
        ssh '"$ssh_options"' root@127.0.0.1 "cat >/mnt/data/supervisor/ssl/memory-card-demo.key" <tls/key.pem'
    docker exec "$name" cat tls/ca.pem >"$state/ca.pem"
}

# Points the DuckDNS name at this machine, renews its certificate when it is
# due and puts it in Home Assistant's ssl folder. Prints whether it changed.
# The token never reaches a command line.
duckdns_certificate() {
    if [ ! -s "$token_file" ]; then
        echo "Put the DuckDNS token in $token_file" >&2
        exit 1
    fi
    answer=$(printf 'url = "https://www.duckdns.org/update?domains=%s&token=%s&ip=%s"\n' \
        "$HA_DUCKDNS" "$(cat "$token_file")" "$(lan_ip)" | curl -fsS -K -)
    if [ "$answer" != OK ]; then
        echo "DuckDNS refused the update. Check the name and $token_file." >&2
        exit 1
    fi
    mkdir -p "$certs"
    [ -f "$certs/config" ] || printf 'BASEDIR=/certs\nKEY_ALGO=prime256v1\n' >"$certs/config"
    if ! log=$(DUCKDNS_TOKEN=$(cat "$token_file") docker run --rm --user "$(id -u):$(id -g)" \
        --cap-drop ALL --security-opt no-new-privileges --read-only --tmpfs /tmp \
        -e DUCKDNS_TOKEN -v "$PWD/$certs:/certs" --entrypoint sh "$image" -c '
            dehydrated --config /certs/config --register --accept-terms &&
                dehydrated --config /certs/config --cron --challenge dns-01 \
                    --hook /usr/local/bin/duckdns-hook.sh --domain "$0"' "$duckdns" 2>&1); then
        echo "$log" >&2
        echo "Let's Encrypt did not issue the certificate" >&2
        exit 1
    fi
    live=$certs/certs/$duckdns
    sum=$(sha256sum <"$live/fullchain.pem")
    if [ "$(cat "$state/certificate" 2>/dev/null)" = "$sum" ]; then
        echo same
        return
    fi
    vm_ssh "cat >/mnt/data/supervisor/ssl/memory-card-demo.pem" <"$live/fullchain.pem"
    vm_ssh "cat >/mnt/data/supervisor/ssl/memory-card-demo.key" <"$live/privkey.pem"
    echo "$sum" >"$state/certificate"
    echo changed
}

# Where Home Assistant answers now: plainly, with the test CA, or with the
# DuckDNS certificate. Nothing while it installs or restarts.
current_url() {
    if curl -fsS -o /dev/null "http://127.0.0.1:$port/" 2>/dev/null; then
        echo "http://127.0.0.1:$port"
    elif [ -f "$state/ca.pem" ] &&
        curl -fsS -o /dev/null --cacert "$state/ca.pem" "https://127.0.0.1:$port/" 2>/dev/null; then
        echo "https://127.0.0.1:$port"
    elif [ -n "$duckdns" ] && curl -fsS -o /dev/null "https://$duckdns:$port/" 2>/dev/null; then
        echo "https://$duckdns:$port"
    else
        return 1
    fi
}

download() {
    [ -f "$os_file" ] && return
    mkdir -p "$(dirname "$os_file")"
    echo "Downloading Home Assistant OS $os_version"
    curl -fL --retry 3 -o "$os_file.part" \
        "https://github.com/home-assistant/operating-system/releases/download/$os_version/haos_ova-$os_version.qcow2.xz"
    if ! echo "$os_sha256  $os_file.part" | sha256sum -c --quiet -; then
        rm -f "$os_file.part"
        echo "The download does not match the expected checksum" >&2
        exit 1
    fi
    mv "$os_file.part" "$os_file"
}

# The Supervisor builds the app from this folder. Its Dockerfile expects the
# repository root as the build context, so the binary and rootfs sit at the
# same paths here. Without `image` the Supervisor builds instead of pulling,
# and a new version each time lets an update apply config.yaml changes.
stage() {
    stamp=$(date +%Y%m%d%H%M%S)
    app=$apps/memory-card
    mkdir -p "$apps"
    rm -rf "$app"
    mkdir -p "$app/dist/amd64" "$app/memory-card"
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
    cp dist/amd64/memory-card "$app/dist/amd64/"
}

# Puts the integration into Home Assistant's config folder. Home Assistant
# only loads a changed integration after a restart, which ha-demo.mjs does.
copy_integration() {
    HA_INTEGRATION_CHANGED=
    source=$integration/custom_components
    [ -d "$source/memory_card" ] || return 0
    sum=$(tar -C "$source" --exclude=__pycache__ --sort=name --mtime=@0 --owner=0 --group=0 \
        -cf - memory_card | sha256sum | cut -c1-64)
    if [ "$(cat "$state/integration.sha256" 2>/dev/null)" = "$sum" ] &&
        vm_ssh test -d /mnt/data/supervisor/homeassistant/custom_components/memory_card </dev/null; then
        return 0
    fi
    echo "Copying the Memory Card integration from $integration"
    tar -C "$source" --exclude=__pycache__ -cf - memory_card | vm_ssh '
        set -e
        mkdir -p /mnt/data/supervisor/homeassistant/custom_components
        cd /mnt/data/supervisor/homeassistant/custom_components
        rm -rf memory_card
        tar -xf -'
    mkdir -p "$state"
    echo "$sum" >"$state/integration.sha256"
    HA_INTEGRATION_CHANGED=1
}

up() {
    if [ "$(uname -m)" != x86_64 ] || [ ! -e /dev/kvm ]; then
        echo "make ha-up needs an x86-64 Linux machine with KVM" >&2
        exit 1
    fi
    refuse_foreign
    stage
    # Without provenance an unchanged build keeps its image ID.
    docker build -q --provenance=false -t "$image" --label "$label=1" scripts/haos >/dev/null
    docker image prune -f --filter "label=$label=1" >/dev/null

    bind=127.0.0.1
    [ -z "${HA_LAN:-}" ] || bind=0.0.0.0
    # A new image or a different address needs a new container; the volume
    # keeps the VM.
    if exists && { [ "$(docker container inspect -f '{{.Image}}' "$name")" != "$(docker image inspect -f '{{.Id}}' "$image")" ] ||
        [ "$(docker container inspect -f '{{range (index .HostConfig.PortBindings "8080/tcp")}}{{.HostIp}}:{{.HostPort}}{{end}}' "$name")" != "$bind:$port" ]; }; then
        echo "Recreating the container"
        docker stop "$name" >/dev/null
        docker rm "$name" >/dev/null
    fi

    if ! exists; then
        if ! docker volume inspect "$name-vm" >/dev/null 2>&1; then
            download
            docker volume create --label "$label=1" "$name-vm" >/dev/null
            echo "Unpacking Home Assistant OS"
            docker run --rm -i --cap-drop ALL -v "$name-vm:/vm" --entrypoint sh "$image" \
                -c 'xz -dc >haos.qcow2.part && mv haos.qcow2.part haos.qcow2' <"$os_file"
        fi
        # The stop timeout lets Home Assistant OS shut down when Docker stops.
        docker run -d --name "$name" --label "$label=1" --stop-timeout 180 \
            --device /dev/kvm --group-add "$(stat -c %g /dev/kvm)" \
            --cap-drop ALL --security-opt no-new-privileges --read-only --tmpfs /tmp \
            -p "$bind:$port:8080" \
            -v "$name-vm:/vm" \
            "$image" >/dev/null
        echo "Starting Home Assistant OS"
    elif ! running; then
        docker start "$name" >/dev/null
        echo "Starting Home Assistant OS"
    fi

    tries=0
    until vm_ssh true </dev/null 2>/dev/null; do
        tries=$((tries + 1))
        if ! running; then
            docker logs --tail 20 "$name" >&2
            exit 1
        fi
        if [ "$tries" -gt 120 ]; then
            echo "Home Assistant OS did not open SSH" >&2
            exit 1
        fi
        sleep 5
    done

    tar -C "$apps" -cf - memory-card | vm_ssh '
        set -e
        mkdir -p /mnt/data/supervisor/apps/local
        cd /mnt/data/supervisor/apps/local
        rm -rf memory-card
        tar -xf -'
    copy_integration

    certificate=
    if [ -n "$duckdns" ]; then
        certificate=$(duckdns_certificate)
        HA_URL=https://$duckdns:$port
    elif [ -n "${HA_HTTPS:-}" ]; then
        certificate=$(issue_certificate)
        HA_URL=https://127.0.0.1:$port
    else
        HA_URL=http://127.0.0.1:$port
    fi
    HA_NEW_CERTIFICATE=
    [ "$certificate" != changed ] || HA_NEW_CERTIFICATE=1
    export HA_URL HA_NEW_CERTIFICATE HA_INTEGRATION_CHANGED

    tries=0
    until current=$(current_url); do
        tries=$((tries + 1))
        if [ "$tries" -gt 600 ]; then
            echo "Home Assistant did not answer" >&2
            exit 1
        fi
        sleep 3
    done
    # The test CA, when there is one, is trusted alongside the usual ones.
    if [ -f "$state/ca.pem" ]; then
        NODE_EXTRA_CA_CERTS=$state/ca.pem node scripts/ha-demo.mjs "$current" "$state/demo.json"
    else
        node scripts/ha-demo.mjs "$current" "$state/demo.json"
    fi

    if [ -n "${HA_LAN:-}" ] && [ -z "$duckdns" ]; then
        echo "On the network it is at ${HA_URL%%://*}://$(lan_ip):$port"
    fi
}

case ${1:-} in
up)
    up
    ;;
stop)
    refuse_foreign
    if running; then
        docker stop "$name" >/dev/null
    fi
    ;;
destroy)
    refuse_foreign
    docker rm -f "$name" >/dev/null 2>&1 || true
    if labelled volume "" "$name-vm"; then
        docker volume rm "$name-vm" >/dev/null
    fi
    if labelled image Config. "$image"; then
        docker image rm "$image" >/dev/null
    fi
    docker image prune -f --filter "label=$label=1" >/dev/null
    rm -rf "$state"
    ;;
logs)
    refuse_foreign
    vm_ssh docker logs -f app_local_memory_card
    ;;
*)
    echo "usage: $0 up|stop|destroy|logs" >&2
    exit 1
    ;;
esac
