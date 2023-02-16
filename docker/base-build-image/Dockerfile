# to specify a different alpine version, use --build-arg ALPINE_VERSION=3.12
# when building the image
ARG ALPINE_VERSION=latest
FROM alpine:${ALPINE_VERSION}
LABEL org.opencontainers.image.title=niumside-base-build-image
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=

RUN apk add --no-cache \
        ca-certificates \
        gcc \
        openssl-dev \
        musl-dev

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    apkArch="$(apk --print-arch)"; \
    case "$apkArch" in \
        x86_64) rustArch='x86_64-unknown-linux-musl' ;; \
        aarch64) rustArch='aarch64-unknown-linux-musl' ;; \
        *) echo >&2 "unsupported architecture: $apkArch"; exit 1 ;; \
    esac; \
    \
    url="https://static.rust-lang.org/rustup/dist/${rustArch}/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain nightly; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

RUN cargo install cargo-chef
