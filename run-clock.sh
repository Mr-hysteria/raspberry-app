#!/usr/bin/env bash

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_BINARY="${PROJECT_DIR}/target/release/raspberry-clock"
DEBUG_BINARY="${PROJECT_DIR}/target/debug/raspberry-clock"

export DISPLAY="${DISPLAY:-:0}"
export XAUTHORITY="${XAUTHORITY:-/home/raspberry/.Xauthority}"
export SLINT_BACKEND="${SLINT_BACKEND:-winit-software}"
export SLINT_FULLSCREEN="${SLINT_FULLSCREEN:-1}"

# 关闭屏保和节能，避免时钟屏幕熄灭。
xset s off || true
xset -dpms || true
xset s noblank || true

# 隐藏鼠标光标。
if command -v unclutter >/dev/null 2>&1; then
    pkill -x unclutter >/dev/null 2>&1 || true
    unclutter --timeout 0 --jitter 1 --fork >/dev/null 2>&1 || true
fi

# 避免重复打开多个时钟窗口。
pkill -f "${RELEASE_BINARY}" >/dev/null 2>&1 || true
pkill -f "${DEBUG_BINARY}" >/dev/null 2>&1 || true

sleep 1

if [[ -x "${RELEASE_BINARY}" ]]; then
    exec "${RELEASE_BINARY}"
fi

if [[ -x "${DEBUG_BINARY}" ]]; then
    exec "${DEBUG_BINARY}"
fi

echo "Binary not found." >&2
echo "Please run: cargo build --release || cargo build" >&2
exit 1
