#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_SCRIPT="${ROOT_DIR}/run-clock.sh"

grep -Fq 'xset dpms 0 0 0 || true' "${RUN_SCRIPT}"
