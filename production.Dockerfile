FROM docker.io/alpine:latest
ARG TARGETPLATFORM
WORKDIR /usr/src/niumside-poptracker
COPY ./config/default.yaml ./config/default.yaml
COPY ./niumside-poptracker-* .

# dynamically delete the binary that doesn't match the target platform
# this is niumside-poptracker-x86_64-unknown-linux-musl for amd64
# and niumside-poptracker-aarch64-unknown-linux-musl for arm64
# Then move the binary that matches the target platform to niumside-poptracker
RUN rm -f niumside-poptracker-$(echo $TARGETPLATFORM | cut -d '/' -f 2)-unknown-linux-musl && \
    mv niumside-poptracker-$(echo $TARGETPLATFORM | cut -d '/' -f 3)-unknown-linux-musl niumside-poptracker

CMD ["./niumside-poptracker"]
