#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WATCH_SCRIPT="${ROOT_DIR}/scripts/watch-clock.sh"

grep -Fq 'while true; do' "${WATCH_SCRIPT}"
grep -Fq '"${PROJECT_DIR}/run-clock.sh"' "${WATCH_SCRIPT}"
grep -Fq 'sleep 3' "${WATCH_SCRIPT}"
grep -Fq 'raspberry-clock-watch.pid' "${WATCH_SCRIPT}"
grep -Fq 'flock -n 9' "${WATCH_SCRIPT}"
