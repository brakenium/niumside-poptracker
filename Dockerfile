FROM ghcr.io/brakenium/base-build-image as planner
WORKDIR /usr/src/niumside-poptracker
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM ghcr.io/brakenium/base-build-image as cacher
WORKDIR /usr/src/niumside-poptracker
COPY --from=planner /usr/src/niumside-poptracker/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM ghcr.io/brakenium/base-build-image as builder
WORKDIR /usr/src/niumside-poptracker
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /usr/src/niumside-poptracker/target target
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
