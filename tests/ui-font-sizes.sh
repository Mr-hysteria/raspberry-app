#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_FILE="${ROOT_DIR}/ui/clock.slint"

grep -Fq 'in-out property <string> date-weekday-text:' "${UI_FILE}"
if grep -Fq 'text: "年度倒计时"' "${UI_FILE}"; then
    echo "redundant annual countdown title still exists" >&2
    exit 1
fi
if grep -Fq 'text: "CPA 考试倒计时"' "${UI_FILE}"; then
    echo "redundant CPA countdown title still exists" >&2
    exit 1
fi
grep -A10 'text: root.year-remaining-text;' "${UI_FILE}" | grep -Fq 'font-size: 28px;'
grep -A10 'text: root.year-remaining-text;' "${UI_FILE}" | grep -Fq 'font-weight: 700;'
grep -A10 'text: root.cpa-countdown-text;' "${UI_FILE}" | grep -Fq 'font-size: 28px;'
grep -A10 'text: root.cpa-countdown-text;' "${UI_FILE}" | grep -Fq 'font-weight: 700;'
grep -A10 'text: root.quote-chinese;' "${UI_FILE}" | grep -Fq 'font-size: 21px;'
grep -Fq 'height: 7px;' "${UI_FILE}"
