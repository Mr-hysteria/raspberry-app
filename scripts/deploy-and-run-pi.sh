#!/usr/bin/env bash

set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

TARGET_TRIPLE="${PI_TARGET_TRIPLE:-aarch64-unknown-linux-gnu}"
BINARY_NAME="${PI_BINARY_NAME:-raspberry-clock}"

REMOTE_USER="${PI_USER:-raspberry}"
REMOTE_SSH_HOST="${PI_SSH_HOST:-raspberry-clock}"
REMOTE_HOST="${PI_HOST:-192.168.1.12}"
REMOTE_PROJECT_DIR="${PI_PROJECT_DIR:-/home/${REMOTE_USER}/Desktop/code/raspberry-app}"
REMOTE_DISPLAY="${PI_DISPLAY:-:0}"
REMOTE_XAUTHORITY="${PI_XAUTHORITY:-/home/${REMOTE_USER}/.Xauthority}"

REMOTE_PASSWORD="${PI_PASSWORD:-}"
REMOTE_GIT_PULL="${PI_REMOTE_GIT_PULL:-0}"

LOCAL_BINARY_PATH="${PROJECT_DIR}/target/${TARGET_TRIPLE}/release/${BINARY_NAME}"
LOCAL_RUN_SCRIPT="${PROJECT_DIR}/run-clock.sh"
LOCAL_BOOTSTRAP_SCRIPT="${PROJECT_DIR}/scripts/bootstrap-pi.sh"

REMOTE_BINARY_DIR="${REMOTE_PROJECT_DIR}/target/release"
REMOTE_BINARY_PATH="${REMOTE_BINARY_DIR}/${BINARY_NAME}"
REMOTE_RUN_SCRIPT="${REMOTE_PROJECT_DIR}/run-clock.sh"
REMOTE_BOOTSTRAP_SCRIPT="${REMOTE_PROJECT_DIR}/scripts/bootstrap-pi.sh"

if [[ -n "${PI_SSH_HOST:-}" ]]; then
    REMOTE_PROJECT_REF="${REMOTE_SSH_HOST}"
else
    REMOTE_PROJECT_REF="${REMOTE_USER}@${REMOTE_HOST}"
fi

ssh_cmd() {
    if [[ -n "${REMOTE_PASSWORD}" ]]; then
        if ! command -v sshpass >/dev/null 2>&1; then
            echo "PI_PASSWORD is set, but sshpass is not installed." >&2
            echo "Install it with: brew install sshpass" >&2
            exit 1
        fi
        sshpass -p "${REMOTE_PASSWORD}" ssh "$@"
    else
        ssh "$@"
    fi
}

scp_cmd() {
    if [[ -n "${REMOTE_PASSWORD}" ]]; then
        if ! command -v sshpass >/dev/null 2>&1; then
            echo "PI_PASSWORD is set, but sshpass is not installed." >&2
            echo "Install it with: brew install sshpass" >&2
            exit 1
        fi
        sshpass -p "${REMOTE_PASSWORD}" scp "$@"
    else
        scp "$@"
    fi
}

echo "==> Cross-compiling ${BINARY_NAME} for ${TARGET_TRIPLE}"
(
    cd "${PROJECT_DIR}"
    PKG_CONFIG_ALLOW_CROSS=1 cargo build --release --target "${TARGET_TRIPLE}"
)

echo "==> Preparing remote directories on ${REMOTE_PROJECT_REF}"
ssh_cmd "${REMOTE_PROJECT_REF}" \
    "mkdir -p '${REMOTE_BINARY_DIR}' '${REMOTE_PROJECT_DIR}/scripts'"

if [[ "${REMOTE_GIT_PULL}" == "1" ]]; then
    echo "==> Fast-forward pulling remote repository"
    ssh_cmd "${REMOTE_PROJECT_REF}" "
        if [ -d '${REMOTE_PROJECT_DIR}/.git' ]; then
            cd '${REMOTE_PROJECT_DIR}'
            git pull --ff-only
        else
            echo 'Remote project is not a git repository, skipping git pull.'
        fi
    "
fi

echo "==> Syncing runtime scripts"
scp_cmd "${LOCAL_RUN_SCRIPT}" "${REMOTE_PROJECT_REF}:${REMOTE_RUN_SCRIPT}"
scp_cmd "${LOCAL_BOOTSTRAP_SCRIPT}" "${REMOTE_PROJECT_REF}:${REMOTE_BOOTSTRAP_SCRIPT}"

echo "==> Uploading binary"
scp_cmd "${LOCAL_BINARY_PATH}" "${REMOTE_PROJECT_REF}:${REMOTE_BINARY_PATH}"

echo "==> Setting executable permissions"
ssh_cmd "${REMOTE_PROJECT_REF}" \
    "chmod +x '${REMOTE_RUN_SCRIPT}' '${REMOTE_BOOTSTRAP_SCRIPT}' '${REMOTE_BINARY_PATH}'"

echo "==> Starting app on Raspberry Pi"
ssh_cmd "${REMOTE_PROJECT_REF}" "
    DISPLAY='${REMOTE_DISPLAY}' XAUTHORITY='${REMOTE_XAUTHORITY}' \
    nohup '${REMOTE_RUN_SCRIPT}' >/tmp/${BINARY_NAME}.log 2>&1 </dev/null &
"

echo
echo "Deployment completed."
echo "Remote host: ${REMOTE_PROJECT_REF}"
echo "Binary path: ${REMOTE_BINARY_PATH}"
echo
echo "To stop the remote app later:"
echo "ssh ${REMOTE_PROJECT_REF} \"pkill -f ${BINARY_NAME}\""
echo
echo "To inspect the latest remote log:"
echo "ssh ${REMOTE_PROJECT_REF} \"tail -n 80 /tmp/${BINARY_NAME}.log\""
