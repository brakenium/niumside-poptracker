FROM --platform=$BUILDPLATFORM docker.io/debian:bullseye-slim
LABEL org.opencontainers.image.title=niumside-base-build-image
# LABEL org.opencontainers.image.description=
LABEL org.opencontainers.image.url=https://github.com/brakenium/niumside-poptracker
LABEL org.opencontainers.image.source=https://github.com/brakenium/niumside-poptracker
# LABEL org.opencontainers.image.licenses=
ARG TARGETPLATFORM
ARG BUILDPLATFORM

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
        libssl-dev \
        pkg-config \
        wget \
        ; \
    case $BUILDPLATFORM in \
        linux/amd64) rustArchBP='x86_64-unknown-linux-gnu' ;; \
        linux/arm64) rustArchBP='aarch64-unknown-linux-gnu' ;; \
        linux/arm/v7) rustArchBP='armv7-unknown-linux-gnueabihf' ;; \
        *) echo >&2 "unsupported architecture: ${BUILDPLATFORM}"; exit 1 ;; \
    esac; \
    \
    case $TARGETPLATFORM in \
        linux/amd64) rustArchTP='x86_64-unknown-linux-gnu' ;; \
        linux/arm64) rustArchTP='aarch64-unknown-linux-gnu' ;; \
        linux/arm/v7) rustArchTP='armv7-unknown-linux-gnueabihf' ;; \
        *) echo >&2 "unsupported architecture: ${TARGETPLATFORM}"; exit 1 ;; \
    esac; \
    \
    url="https://static.rust-lang.org/rustup/dist/${rustArchBP}/rustup-init"; \
    wget "$url"; \
    chmod +x rustup-init; \
    ./rustup-init -y --no-modify-path --default-toolchain nightly --target $rustArchTP; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version; \
    \
    apt-get remove -y --auto-remove \
        wget \
        ; \
    rm -rf /var/lib/apt/lists/*;

RUN cargo install cargo-chef
