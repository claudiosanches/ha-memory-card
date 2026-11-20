# Build context is the repository root. The binaries are built outside Docker
# (cargo zigbuild) into dist/<docker arch>/, so this image only copies files.
FROM ghcr.io/home-assistant/base:3.24

ARG TARGETARCH
ARG BUILD_VERSION=dev

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
    org.opencontainers.image.licenses="GPL-3.0-only" \
    org.opencontainers.image.version="${BUILD_VERSION}"
