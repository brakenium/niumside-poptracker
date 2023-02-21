#!/bin/bash
set -eux

BRANCH_NAME=$(git rev-parse --abbrev-ref HEAD)

if [ "$BRANCH_NAME" = "master" ]; then
  BRANCH_NAME="latest"
fi

pushd docker/base-build-image
docker buildx build --cache-from type=registry,ref="ghcr.io/brakenium/base-build-image:$BRANCH_NAME" --cache-from type=registry,ref="ghcr.io/brakenium/base-build-image" --cache-to type=inline --platform linux/amd64,linux/arm64,linux/arm/v7 --tag "ghcr.io/brakenium/base-build-image:$BRANCH_NAME" --push .

popd
docker buildx build --cache-from type=registry,ref="ghcr.io/brakenium/niumside-poptracker:$BRANCH_NAME" --cache-from type=registry,ref="ghcr.io/brakenium/base-build-image" --cache-to type=inline --platform linux/amd64,linux/arm64,linux/arm/v7 --tag "ghcr.io/brakenium/niumside-poptracker:$BRANCH_NAME" --push .
