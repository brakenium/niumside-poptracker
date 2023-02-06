FROM docker.io/rustlang/rust:nightly-alpine as builder
WORKDIR /usr/src/
RUN apk add --no-cache openssl-dev musl-dev
WORKDIR /usr/src/niumside-poptracker
COPY ./Cargo.toml .
COPY ./src/main.rs ./src/
COPY ./auraxis-rs/auraxis/Cargo.toml ./auraxis-rs/auraxis/Cargo.toml
# TODO: Find out why this can not find auraxis_macros
# RUN cargo fetch
COPY . .
RUN cargo build --release

FROM docker.io/alpine:latest
WORKDIR /usr/src/niumside-poptracker
COPY --from=builder /usr/src/niumside-poptracker/target/release/niumside-poptracker .
COPY --from=builder /usr/src/niumside-poptracker/config ./config
CMD ["./niumside-poptracker"]
