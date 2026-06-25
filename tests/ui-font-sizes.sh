#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UI_FILE="${ROOT_DIR}/ui/clock.slint"

grep -A8 'text: "年度倒计时"' "${UI_FILE}" | grep -Fq 'font-size: 17px;'
grep -A8 'text: "CPA 考试倒计时"' "${UI_FILE}" | grep -Fq 'font-size: 17px;'
grep -A8 'text: root.cpa-date-text;' "${UI_FILE}" | grep -Fq 'font-size: 16px;'
grep -A8 'text: root.quote-chinese;' "${UI_FILE}" | grep -Fq 'font-size: 24px;'
