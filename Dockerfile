FROM rustlang/rust:nightly-alpine as builder
WORKDIR /usr/src/
RUN apk add --no-cache git openssl-dev musl-dev
RUN git clone https://github.com/brakenium/auraxis-rs.git
WORKDIR /usr/src/niumside-poptracker
COPY Cargo.toml .
COPY src/main.rs ./src/
RUN cargo fetch
COPY . .
RUN cargo build --release

FROM alpine:latest
WORKDIR /usr/src/niumside-poptracker
COPY --from=builder /usr/src/niumside-poptracker/target/release/niumside-poptracker .
COPY --from=builder /usr/src/niumside-poptracker/config ./config
CMD ["./niumside-poptracker"]
