# Build context is the repository root. The binaries are built outside Docker
# (cargo zigbuild) into dist/<docker arch>/, so this image only copies files.
# Pinned by digest so a rebuild uses the image that was tested; Dependabot
# proposes new digests.
FROM ghcr.io/home-assistant/base:3.24@sha256:5021aac8a4b7f2a1397e23a9749dfe9f736463ffa471df6b200216c68d530fe6

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
    org.opencontainers.image.description="Track your video game collection across consoles" \
    org.opencontainers.image.source="https://github.com/claudiosanches/ha-memory-card" \
    org.opencontainers.image.licenses="Apache-2.0" \
    org.opencontainers.image.version="${BUILD_VERSION}"
