#!/usr/bin/env sh
NIUMSIDE_BASE_URL="http://localhost:8000"

timestamp=$(curl -s -X GET "${NIUMSIDE_BASE_URL}/api/population" | jq -r '.pop.timestamp')
timestampMillis=$(date -d "${timestamp}" -u +%s%3N)
currentMillis=$(date +%s%3N)
# shellcheck disable=SC2004
difference=$(($currentMillis - $timestampMillis))

if [ $difference -lt 60000 ]; then
  echo "true"
else
  echo "false"
  exit 1
fi
