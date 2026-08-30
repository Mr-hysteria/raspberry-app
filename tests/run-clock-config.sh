#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_SCRIPT="${ROOT_DIR}/run-clock.sh"
TEMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TEMP_DIR}"' EXIT

grep -Fq 'xset dpms 0 0 0 || true' "${RUN_SCRIPT}"
grep -Fq 'unclutter -idle 1 -jitter 1 -root' "${RUN_SCRIPT}"

if grep -Eq 'unclutter .*--(timeout|jitter|fork)' "${RUN_SCRIPT}"; then
    echo "run-clock.sh uses unclutter-xfixes flags with classic unclutter" >&2
    exit 1
fi

mkdir -p "${TEMP_DIR}/bin" "${TEMP_DIR}/target/release"
cp "${RUN_SCRIPT}" "${TEMP_DIR}/run-clock.sh"

printf '%s\n' \
    '#!/usr/bin/env bash' \
    'printf "%s %s\n" "${0##*/}" "$*" >>"${TRACE_FILE}"' \
    >"${TEMP_DIR}/bin/mock-command"
chmod +x "${TEMP_DIR}/bin/mock-command"

for command_name in xrandr xset unclutter pkill sleep; do
    ln -s mock-command "${TEMP_DIR}/bin/${command_name}"
done

printf '%s\n' \
    '#!/usr/bin/env bash' \
    'printf "raspberry-clock\n" >>"${TRACE_FILE}"' \
    >"${TEMP_DIR}/target/release/raspberry-clock"
chmod +x "${TEMP_DIR}/target/release/raspberry-clock"

TRACE_FILE="${TEMP_DIR}/trace.log" \
PATH="${TEMP_DIR}/bin:/usr/bin:/bin" \
DISPLAY=:99 \
XAUTHORITY="${TEMP_DIR}/.Xauthority" \
bash "${TEMP_DIR}/run-clock.sh"

grep -Fxq 'xrandr --output HDMI-1 --set Broadcast RGB Full' "${TEMP_DIR}/trace.log"
