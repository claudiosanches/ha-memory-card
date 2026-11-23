# Build context is the repository root. The binaries are built outside Docker
# (cargo zigbuild) into dist/<docker arch>/, so this image only copies files.
# Pinned by digest so a rebuild uses the image that was tested; Dependabot
# proposes new digests.
FROM ghcr.io/home-assistant/base:3.24@sha256:58422480a299b16b6ea376c2167463206d430619ed02b40cd687c8bd6875519e

ARG TARGETARCH
ARG BUILD_VERSION=dev

# The server runs as this user; a fixed id keeps the files in /data its own
# across image updates.
RUN addgroup -S -g 1000 memory-card \
    && adduser -S -D -H -h /data -s /sbin/nologin -u 1000 -G memory-card memory-card

COPY --chmod=755 dist/${TARGETARCH}/memory-card /usr/bin/memory-card
COPY memory-card/rootfs /

LABEL \
    io.hass.type="app" \
    io.hass.name="Memory Card" \
    io.hass.description="Track your video game collection across consoles" \
    io.hass.url="https://github.com/claudiosanches/ha-memory-card" \
    io.hass.version="${BUILD_VERSION}" \
    org.opencontainers.image.title="Memory Card" \
    org.opencontainers.image.source="https://github.com/claudiosanches/ha-memory-card" \
    org.opencontainers.image.licenses="GPL-3.0-or-later" \
    org.opencontainers.image.version="${BUILD_VERSION}"
