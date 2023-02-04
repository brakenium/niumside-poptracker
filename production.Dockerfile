FROM docker.io/alpine:latest
ARG TARGETPLATFORM
WORKDIR /usr/src/niumside-poptracker
COPY ./config/default.yaml ./config/default.yaml
COPY ./niumside-poptracker-* .

RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
        rm niumside-poptracker-aarch64-unknown-linux-musl; \
        mv niumside-poptracker-x86_64-unknown-linux-musl niumside-poptracker; \
    else \
        rm niumside-poptracker-x86_64-unknown-linux-musl; \
        mv niumside-poptracker-aarch64-unknown-linux-musl niumside-poptracker; \
    fi


CMD ["./niumside-poptracker"]
