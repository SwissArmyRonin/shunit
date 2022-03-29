#!/usr/bin/env bash

set -euo pipefail

echo "Starting" >&2
for ((i = 0; i <= 5; i++)); do
    sleep 1
    echo "$i"
done
echo "Done" >&2
