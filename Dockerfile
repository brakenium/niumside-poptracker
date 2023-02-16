FROM ghcr.io/brakenium/niumside-base-build-image as builder
LABEL org.opencontainers.image.title=niumside-poptracker
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

WORKDIR /usr/src/niumside-poptracker
COPY . .
RUN cargo build --release --bin niumside-poptracker

CMD ["./target/release/niumside-poptracker"]
