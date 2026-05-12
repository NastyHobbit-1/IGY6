#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"
DEFAULT_DATA_ROOT="${REPO_ROOT}/../IGY6_Data"

usage() {
  cat <<'EOF'
Usage:
  scripts/stop-host-bridge.sh [--help]

Stops the local IGY6 Rust host bridge started by scripts/start-host-bridge.sh.
Persistent data and Docker resources are not touched.
EOF
}

for arg in "$@"; do
  case "${arg}" in
    --help|-h) usage; exit 0 ;;
    *) printf 'ERROR: unknown argument: %s\n' "${arg}" >&2; exit 1 ;;
  esac
done

env_value() {
  local key="$1"
  if [[ -f "${ENV_FILE}" ]]; then
    awk -F= -v key="${key}" '$1 == key { print substr($0, length(key) + 2); exit }' "${ENV_FILE}"
  fi
}

resolve_data_root() {
  local configured="${IGY6_DATA_ROOT:-$(env_value IGY6_DATA_ROOT)}"
  if [[ -z "${configured}" ]]; then
    printf '%s\n' "${DEFAULT_DATA_ROOT}"
    return
  fi
  if [[ "${configured}" = /* || "${configured}" =~ ^[A-Za-z]:/ ]]; then
    printf '%s\n' "${configured}"
  else
    printf '%s\n' "${REPO_ROOT}/${configured}"
  fi
}

DATA_ROOT="$(resolve_data_root)"
PID_FILE="${DATA_ROOT}/ops/host-bridge.pid"

if [[ ! -f "${PID_FILE}" ]]; then
  printf 'Host bridge PID file not found. Nothing to stop.\n'
  exit 0
fi

PID="$(cat "${PID_FILE}")"
if [[ -z "${PID}" ]]; then
  rm -f "${PID_FILE}"
  printf 'Host bridge PID file was empty and has been removed.\n'
  exit 0
fi

if kill -0 "${PID}" >/dev/null 2>&1; then
  kill "${PID}"
  printf 'Stopped host bridge PID %s.\n' "${PID}"
else
  printf 'Host bridge PID %s is not running.\n' "${PID}"
fi

rm -f "${PID_FILE}"
