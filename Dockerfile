FROM ghcr.io/brakenium/base-build-image as builder
WORKDIR /usr/src/niumside-poptracker
COPY . .
RUN cargo build --release --bin niumside-poptracker

FROM docker.io/alpine as runtime
LABEL org.opencontainers.image.title=niumside-poptracker
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

WORKDIR /usr/src/niumside-poptracker
COPY --from=builder /usr/src/niumside-poptracker/target/release/niumside-poptracker ./niumside-poptracker
COPY --from=builder /usr/src/niumside-poptracker/config ./config
CMD ["./niumside-poptracker"]
