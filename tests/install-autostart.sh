#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TEMP_DIR}"' EXIT

AUTOSTART_DIR="${TEMP_DIR}/autostart" \
    "${ROOT_DIR}/scripts/install-autostart.sh"

DESKTOP_FILE="${TEMP_DIR}/autostart/raspberry-clock.desktop"
test -f "${DESKTOP_FILE}"
grep -Fq '[Desktop Entry]' "${DESKTOP_FILE}"
grep -Fq 'Type=Application' "${DESKTOP_FILE}"
grep -Fq "Exec=${ROOT_DIR}/scripts/watch-clock.sh" "${DESKTOP_FILE}"
grep -Fq 'X-GNOME-Autostart-enabled=true' "${DESKTOP_FILE}"
grep -Fq 'Terminal=false' "${DESKTOP_FILE}"
