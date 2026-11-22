#!/bin/sh
# Boots Home Assistant OS from /vm/haos.qcow2 and shuts it down cleanly on
# `docker stop`.
set -eu
cd /vm

if [ ! -f haos.qcow2 ]; then
    echo "No disk in the volume. Run make ha-destroy and make ha-up." >&2
    exit 1
fi
[ -f efivars.fd ] || cp /usr/share/OVMF/OVMF_VARS_4M.fd efivars.fd
if [ ! -f .ssh/id_ed25519 ]; then
    mkdir -p -m 700 .ssh
    ssh-keygen -q -t ed25519 -N '' -C ha-up -f .ssh/id_ed25519
fi
if [ ! -f config.img ]; then
    # Home Assistant OS imports authorized_keys from a drive labelled CONFIG
    # and then opens its developer SSH on port 22222.
    mkfs.vfat -C -n CONFIG config.img.part 8192 >/dev/null
    mcopy -i config.img.part .ssh/id_ed25519.pub ::authorized_keys
    mv config.img.part config.img
fi

# Home Assistant OS ignores the ACPI power button, so ask it over SSH.
qemu=
powerdown() {
    for command in 'ha host shutdown' poweroff; do
        if ssh -p 22222 -o BatchMode=yes -o ConnectTimeout=5 -o StrictHostKeyChecking=accept-new \
            -o LogLevel=ERROR root@127.0.0.1 "$command" >/dev/null 2>&1; then
            return
        fi
    done
    kill "$qemu" 2>/dev/null || true
}
trap powerdown TERM INT

qemu-system-x86_64 \
    -machine q35,accel=kvm -cpu host -smp 2 -m 4096 \
    -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd \
    -drive if=pflash,format=raw,file=efivars.fd \
    -drive if=virtio,format=qcow2,file=haos.qcow2 \
    -drive if=none,id=config,format=raw,file=config.img \
    -device qemu-xhci -device usb-storage,drive=config \
    -netdev user,id=net,hostfwd=tcp::8080-:80,hostfwd=tcp:127.0.0.1:22222-:22222 \
    -device virtio-net-pci,netdev=net \
    -device virtio-rng-pci \
    -display none -serial stdio -monitor none &
qemu=$!

while kill -0 "$qemu" 2>/dev/null; do
    wait "$qemu" || true
done
