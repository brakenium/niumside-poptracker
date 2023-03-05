#!/bin/bash
set -eux

BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)

if [ "$BRANCH_NAME" = "master" ]; then
  BRANCH_NAME="latest"
fi

docker buildx build --cache-from type=registry,ref="ghcr.io/brakenium/niumside-poptracker:$BRANCH_NAME" --cache-from type=registry,ref="ghcr.io/brakenium/niumside-poptracker" --cache-to type=inline --platform linux/amd64,linux/arm64,linux/arm/v7 --tag "ghcr.io/brakenium/niumside-poptracker:$BRANCH_NAME" .
