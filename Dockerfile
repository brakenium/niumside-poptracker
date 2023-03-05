FROM --platform=$TARGETPLATFORM docker.io/debian:bullseye-slim as runtime
ARG TARGETPLATFORM
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

CMD ["niumside-poptracker"]
