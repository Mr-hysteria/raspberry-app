#!/usr/bin/env bash

set -u

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PID_FILE="/tmp/raspberry-clock-watch.pid"
LOCK_FILE="/tmp/raspberry-clock-watch.lock"

exec 9>"${LOCK_FILE}"
flock -n 9 || exit 0

echo "$$" >"${PID_FILE}"
cleanup() {
    rm -f "${PID_FILE}"
}
trap cleanup EXIT
trap 'exit 0' INT TERM

while true; do
    "${PROJECT_DIR}/run-clock.sh"
    exit_code=$?
    echo "raspberry-clock exited with status ${exit_code}; restarting in 3 seconds" >&2
    sleep 3
done
