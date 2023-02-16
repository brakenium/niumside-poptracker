FROM ghcr.io/brakenium/niumside-base-build-image as builder
LABEL org.opencontainers.image.title=niumside-poptracker
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

WORKDIR /usr/src/niumside-poptracker
COPY . .
RUN cargo build --release --bin niumside-poptracker \
    && cp ./target/release/niumside-poptracker ./niumside-poptracker \
    && rm -rf target \
    && rm -rf /usr/local/cargo \
    && rm -rf /usr/local/rustup

CMD ["./niumside-poptracker"]
