# QEMU for the Home Assistant OS virtual machine of `make ha-up`.
FROM debian:trixie-slim@sha256:a72b4433de1cf7d6bae0c4cee74d0547f58601a643b3375f070c51600d63b6b2

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        dosfstools mtools openssh-client ovmf qemu-system-x86 xz-utils \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --home-dir /vm --no-create-home vm \
    && mkdir /vm && chown vm:vm /vm

COPY vm.sh /usr/local/bin/vm.sh
USER vm
WORKDIR /vm
ENTRYPOINT ["/usr/local/bin/vm.sh"]
