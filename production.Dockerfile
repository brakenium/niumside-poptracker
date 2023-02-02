FROM docker.io/alpine:latest
WORKDIR /usr/src/niumside-poptracker
COPY . .

CMD ["./niumside-poptracker"]
