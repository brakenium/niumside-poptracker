FROM docker.io/rustlang/rust:nightly-alpine as planner
WORKDIR /usr/src/niumside-poptracker
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM docker.io/rustlang/rust:nightly-alpine as cacher
WORKDIR /usr/src/niumside-poptracker
RUN apk add --no-cache openssl-dev musl-dev
RUN cargo install cargo-chef
COPY --from=planner /usr/src/niumside-poptracker/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM docker.io/rustlang/rust:nightly-alpine as builder
WORKDIR /usr/src/niumside-poptracker
RUN apk add --no-cache openssl-dev musl-dev
COPY . .
# Copy over the cached dependencies
COPY --from=cacher /usr/src/niumside-poptracker/target target
RUN cargo build --release --bin niumside-poptracker

FROM docker.io/alpine as runtime
WORKDIR /usr/src/niumside-poptracker
COPY --from=builder /usr/src/niumside-poptracker/target/release/niumside-poptracker /niumside-poptracker
COPY --from=builder /usr/src/niumside-poptracker/config ./config
ENTRYPOINT ["./niumside-poptracker"]
