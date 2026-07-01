#!/usr/bin/env bash

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RELEASE_BINARY="${PROJECT_DIR}/target/release/raspberry-clock"
DEBUG_BINARY="${PROJECT_DIR}/target/debug/raspberry-clock"

export DISPLAY="${DISPLAY:-:0}"
export XAUTHORITY="${XAUTHORITY:-/home/raspberry/.Xauthority}"
export SLINT_BACKEND="${SLINT_BACKEND:-winit-software}"
export SLINT_FULLSCREEN="${SLINT_FULLSCREEN:-1}"

# 关闭桌面环境的自动屏保，但保留 DPMS 供应用按时息屏。
xset s off || true
xset +dpms || true
xset dpms 0 0 0 || true
xset s noblank || true

# 隐藏鼠标光标。
if command -v unclutter >/dev/null 2>&1; then
    pkill -x unclutter >/dev/null 2>&1 || true
    unclutter -idle 1 -jitter 1 -root >/dev/null 2>&1 &
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
