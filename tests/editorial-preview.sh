#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TEMP_DIR}"' EXIT

PPM_PATH="${TEMP_DIR}/reading-day.ppm"

cargo run --quiet --manifest-path "${ROOT_DIR}/Cargo.toml" \
    --example render-preview -- day "${PPM_PATH}"

pixel_rgb() {
    local x=$1
    local y=$2
    local offset

    # `write_ppm` emits a 15-byte header for 800x480, followed by RGB bytes.
    offset=$((15 + 3 * (y * 800 + x)))
    dd if="${PPM_PATH}" bs=1 skip="${offset}" count=3 2>/dev/null | od -An -tu1 | xargs
}

read -r surface_r surface_g surface_b <<<"$(pixel_rgb 730 400)"
read -r canvas_r canvas_g canvas_b <<<"$(pixel_rgb 10 10)"

surface_sum=$((surface_r + surface_g + surface_b))
canvas_sum=$((canvas_r + canvas_g + canvas_b))

if (( surface_sum < canvas_sum + 20 )); then
    echo "reading surface is not visibly lifted above the canvas" >&2
    exit 1
fi
