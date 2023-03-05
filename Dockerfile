FROM docker.io/alpine as runtime
LABEL org.opencontainers.image.title=niumside-poptracker
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

WORKDIR /etc/niumside-poptracker/

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates
FROM rust:latest

# Determine the Docker container's architecture and whether it uses musl or glibc
RUN ARCH=$(uname -m); \
    if ldd /bin/sh | grep -q musl; then \
      LIBC="musl"; \
    elif getconf GNU_LIBC_VERSION >/dev/null 2>&1; then \
      LIBC="gnu"; \
    else \
      echo "Error: unknown libc"; \
      exit 1; \
    fi; \
    case "${ARCH}-${LIBC}" in \
      x86_64-gnu) \
        TARGET="x86_64-unknown-linux-gnu"; \
        ;; \
      x86_64-musl) \
        TARGET="x86_64-unknown-linux-musl"; \
        ;; \
      aarch64-gnu) \
        TARGET="aarch64-unknown-linux-gnu"; \
        ;; \
      aarch64-musl) \
        TARGET="aarch64-unknown-linux-musl"; \
        ;; \
      armv7-gnueabihf) \
        TARGET="armv7-unknown-linux-gnueabihf"; \
        ;; \
      armv7-musleabi) \
        TARGET="armv7-unknown-linux-musleabi"; \
        ;; \
      *) \
        echo "Error: unknown architecture or libc: ${ARCH}-${LIBC}"; \
        exit 1; \
        ;; \
    esac; \
    echo "Selected Rust target: ${TARGET}"; \
    export RUST_TARGET=${TARGET}

# Copy binary file based on the selected Rust target
COPY binaries/${RUST_TARGET}/niumside-poptracker /usr/local/bin/niumside-poptracker

CMD ["niumside-poptracker"]
