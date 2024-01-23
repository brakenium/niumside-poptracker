#!/usr/bin/env sh
NIUMSIDE_BASE_URL="https://niumside.brakenium.xyz"

timestamp=$(curl -s -X GET "${NIUMSIDE_BASE_URL}/api/population" | jq -r '.pop.timestamp')
timestampMillis=$(date -d "${timestamp}" -u +%s%3N)
currentMillis=$(date +%s%3N)
# shellcheck disable=SC2004
difference=$(($currentMillis - $timestampMillis))

echo "timestamp: ${timestamp}"
echo "timestampMillis: ${timestampMillis}"
echo "currentMillis: ${currentMillis}"
echo "difference: ${difference}"

if [ $difference -lt 60000 ]; then
  echo "true"
else
  echo "false"
fi
