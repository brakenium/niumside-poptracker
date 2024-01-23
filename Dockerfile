FROM docker.io/alpine as runtime
LABEL org.opencontainers.image.title=niumside-poptracker
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

EXPOSE 8000

HEALTHCHECK --interval=2m --timeout=1m --start-period=45s \
               CMD /etc/niumside-poptracker/check-niumside-health.sh || bash -c 'kill -s 15 -1 && (sleep 10; kill -s 9 -1)'

WORKDIR /etc/niumside-poptracker/

COPY . .

RUN apk add --no-cache curl jq

# Determine the Docker container's architecture and whether it uses musl or glibc
RUN set -eux; \
    ARCH=$(uname -m); \
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
      armv7l-gnu) \
        TARGET="armv7-unknown-linux-gnueabihf"; \
        ;; \
      armv7l-musl) \
        TARGET="armv7-unknown-linux-musleabi"; \
        ;; \
      *) \
        echo "Error: unknown architecture or libc: ${ARCH}-${LIBC}"; \
        exit 1; \
        ;; \
    esac; \
    echo "Selected Rust target: ${TARGET}"; \
    ls -lR; \
    mv binaries/${TARGET}-niumside-poptracker/niumside-poptracker /usr/local/bin/niumside-poptracker; \
    rm -rf binaries; \
    chmod +x /usr/local/bin/niumside-poptracker

CMD ["niumside-poptracker"]
