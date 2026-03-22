#!/usr/bin/env bash

set -euo pipefail

export DEBIAN_FRONTEND=noninteractive

if ! command -v sudo >/dev/null 2>&1; then
    echo "sudo is required" >&2
    exit 1
fi

sudo apt-get update
sudo apt-get install -y \
    build-essential \
    curl \
    libfontconfig1-dev \
    libx11-dev \
    libx11-xcb-dev \
    libxcursor-dev \
    libxkbcommon-x11-dev \
    pkg-config \
    xinput

export RUSTUP_DIST_SERVER="${RUSTUP_DIST_SERVER:-https://rsproxy.cn}"
export RUSTUP_UPDATE_ROOT="${RUSTUP_UPDATE_ROOT:-https://rsproxy.cn/rustup}"

if [[ ! -x "${HOME}/.cargo/bin/rustup" ]]; then
    curl --proto '=https' --tlsv1.2 -sSf https://rsproxy.cn/rustup-init.sh | sh -s -- -y --profile minimal --default-toolchain stable
fi

# shellcheck disable=SC1091
source "${HOME}/.cargo/env"

rustup toolchain install stable --profile minimal
rustup default stable

if ! cargo build --release; then
    cargo build
fi
