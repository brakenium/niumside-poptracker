#!/usr/bin/env sh
sleep 120
while true; do
    timestamp=$(curl -s -X GET "${NIUMSIDE_BASE_URL}/api/population" | jq -r '.pop.timestamp')
    timestampMillis=$(date -d "${timestamp}" -u +%s%3N)
    currentMillis=$(date +%s%3N)
    difference=$(($currentMillis - $timestampMillis))

    echo "Timestamp string: ${timestamp}"
    echo "Current timestamp: ${currentMillis}"
    echo "Population timestamp: ${timestampMillis}"
    echo "Time difference: ${difference}"

    if [ $difference -lt "$MILISECONDS_THRESHOLD" ]; then
      echo "Time difference is within limits"
    else
      echo "Time difference is too large"
      echo "Restarting container ${CONTAINER_TO_WATCH}"
      docker restart "$CONTAINER_TO_WATCH"
      sleep 120
    fi
    sleep "$INTERVAL_SECONDS"
done
