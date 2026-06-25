#!/usr/bin/env bash

set -u

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PID_FILE="/tmp/raspberry-clock-watch.pid"

if [[ -f "${PID_FILE}" ]]; then
    existing_pid="$(cat "${PID_FILE}" 2>/dev/null || true)"
    if [[ -n "${existing_pid}" ]] && kill -0 "${existing_pid}" 2>/dev/null; then
        exit 0
    fi
fi

echo "$$" >"${PID_FILE}"
trap 'rm -f "${PID_FILE}"' EXIT INT TERM

while true; do
    "${PROJECT_DIR}/run-clock.sh"
    exit_code=$?
    echo "raspberry-clock exited with status ${exit_code}; restarting in 3 seconds" >&2
    sleep 3
done
