#!/usr/bin/env bash

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
AUTOSTART_DIR="${AUTOSTART_DIR:-${HOME}/.config/autostart}"
DESKTOP_FILE="${AUTOSTART_DIR}/raspberry-clock.desktop"

mkdir -p "${AUTOSTART_DIR}"

{
    echo "[Desktop Entry]"
    echo "Type=Application"
    echo "Name=Raspberry Clock"
    echo "Comment=Start the full-screen Raspberry Clock after desktop login"
    echo "Exec=${PROJECT_DIR}/run-clock.sh"
    echo "Terminal=false"
    echo "X-GNOME-Autostart-enabled=true"
    echo "StartupNotify=false"
} >"${DESKTOP_FILE}"

chmod 0644 "${DESKTOP_FILE}"

echo "Installed desktop autostart: ${DESKTOP_FILE}"
