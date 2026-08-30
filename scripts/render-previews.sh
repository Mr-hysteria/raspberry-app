#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: scripts/render-previews.sh OUTPUT_DIR" >&2
  exit 1
fi

output_dir=$1
mkdir -p "$output_dir"

tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/raspberry-clock-previews.XXXXXX")
cleanup() {
  rm -rf "$tmp_dir"
}
trap cleanup EXIT

convert_ppm_to_png() {
  local input_path=$1
  local output_path=$2

  if command -v sips >/dev/null 2>&1; then
    sips -s format png "$input_path" --out "$output_path" >/dev/null
    return
  fi

  if command -v convert >/dev/null 2>&1; then
    convert "$input_path" "$output_path"
    return
  fi

  echo "missing image converter: need sips or ImageMagick convert" >&2
  exit 1
}

validate_dimensions() {
  local image_path=$1
  local dimensions

  if command -v sips >/dev/null 2>&1; then
    dimensions=$(sips -g pixelWidth -g pixelHeight "$image_path" 2>/dev/null | awk '/pixelWidth:/ { width=$2 } /pixelHeight:/ { height=$2 } END { if (width == "" || height == "") exit 1; print width "x" height }')
  elif command -v identify >/dev/null 2>&1; then
    dimensions=$(identify -format '%wx%h' "$image_path")
  else
    echo "missing dimension tool: need sips or ImageMagick identify" >&2
    exit 1
  fi

  if [[ "$dimensions" != "800x480" ]]; then
    echo "unexpected dimensions for $image_path: $dimensions" >&2
    exit 1
  fi

  printf '%s\n' "$dimensions"
}

render_state() {
  local state=$1
  local ppm_path=$2
  local png_path=$3
  local dimensions

  cargo run --quiet --example render-preview -- "$state" "$ppm_path"
  convert_ppm_to_png "$ppm_path" "$png_path"
  dimensions=$(validate_dimensions "$png_path")
  printf 'generated %s (%s)\n' "$png_path" "$dimensions"
}

render_state day "$tmp_dir/reading-day.ppm" "$output_dir/reading-day.png"
render_state focus "$tmp_dir/reading-focus.ppm" "$output_dir/reading-focus.png"
render_state night "$tmp_dir/reading-night.ppm" "$output_dir/reading-night.png"
