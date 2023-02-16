FROM ghcr.io/brakenium/base-build-image as builder
LABEL org.opencontainers.image.title=niumside-poptracker
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

WORKDIR /usr/src/niumside-poptracker
# install libssl-dev and pkg-config
RUN apt-get update && apt-get install -y \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release --bin niumside-poptracker

CMD ["./target/release/niumside-poptracker"]
