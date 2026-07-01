#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_SCRIPT="${ROOT_DIR}/run-clock.sh"

grep -Fq 'xset dpms 0 0 0 || true' "${RUN_SCRIPT}"
grep -Fq 'unclutter -idle 1 -jitter 1 -root' "${RUN_SCRIPT}"

if grep -Eq 'unclutter .*--(timeout|jitter|fork)' "${RUN_SCRIPT}"; then
    echo "run-clock.sh uses unclutter-xfixes flags with classic unclutter" >&2
    exit 1
fi
