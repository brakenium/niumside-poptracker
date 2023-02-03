FROM docker.io/alpine:latest
WORKDIR /usr/src/niumside-poptracker
COPY ./config/default.yaml ./config/default.yaml
COPY ./niumside-poptracker ./niumside-poptracker

CMD ["./niumside-poptracker"]
