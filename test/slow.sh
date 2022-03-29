#!/usr/bin/env bash

set -euo pipefail

for ((i = 0; i <= 5; i++)); do
    sleep 1
    echo "$i"
done
