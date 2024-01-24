FROM docker.io/alpine as runtime
LABEL org.opencontainers.image.title=niumside-poptracker-watchdog
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

ENV NIUMSIDE_BASE_URL="http://niumside-poptracker:8000"
ENV MILISECONDS_THRESHOLD=60000
ENV INTERVAL_SECONDS=10
ENV CONTAINER_TO_WATCH=niumside-poptracker

WORKDIR /etc/niumside-watchdog/

ADD --chmod=755 watchdog.sh .

RUN apk add --no-cache curl jq coreutils docker-cli

CMD ["sh", "-c", "./watchdog.sh"]
