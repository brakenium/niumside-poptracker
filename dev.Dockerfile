# This image will build the niumside-poptracker binary from source
FROM docker.io/rust as builder

# Set the working directory

WORKDIR /usr/src/niumside-poptracker

# Copy the source code

COPY . .

# Build the binary

RUN cargo build

# This image will run the niumside-poptracker binary

FROM docker.io/debian

# Set the working directory

WORKDIR /etc/niumside-poptracker

COPY config/ config/

# Copy the binary from the builder image

COPY --from=builder /usr/src/niumside-poptracker/target/debug/niumside-poptracker /usr/local/bin/niumside-poptracker

# Run the binary

CMD ["niumside-poptracker"]
