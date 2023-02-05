FROM docker.io/alpine:latest
ARG TARGETPLATFORM
WORKDIR /usr/src/niumside-poptracker
COPY ./config/default.yaml ./config/default.yaml
COPY ./niumside-poptracker-* .

RUN if [ "$TARGETPLATFORM" = "linux/amd64" ]; then \
        rm niumside-poptracker-aarch64-unknown-linux-musl; \
        mv niumside-poptracker-x86_64-unknown-linux-musl niumside-poptracker; \
        echo "amd64"; \
    else \
        rm niumside-poptracker-x86_64-unknown-linux-musl; \
        mv niumside-poptracker-aarch64-unknown-linux-musl niumside-poptracker; \
        echo "arm64"; \
    fi

# Run in user mode
RUN addgroup -S niumside && adduser -S niumside -G niumside
RUN chown -R niumside:niumside /usr/src/niumside-poptracker && chmod u+x niumside-poptracker
USER niumside

CMD ["./niumside-poptracker"]
