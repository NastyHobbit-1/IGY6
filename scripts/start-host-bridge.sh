#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"
DEFAULT_DATA_ROOT="${REPO_ROOT}/../IGY6_Data"
HOST="127.0.0.1"
PORT="${IGY6_HOST_BRIDGE_PORT:-8765}"

usage() {
  cat <<'EOF'
Usage:
  scripts/start-host-bridge.sh [--foreground|--help]

Starts the local-only Rust host control bridge on 127.0.0.1.

Token handling:
  - Uses IGY6_HOST_BRIDGE_TOKEN_FILE if set.
  - Otherwise creates/uses ${IGY6_DATA_ROOT}/ops/host-bridge.token.
  - The token is not printed.

The bridge executes only fixed IGY6 operator actions and requires
Authorization: Bearer <token> on every request.
EOF
}

FOREGROUND=false
for arg in "$@"; do
  case "${arg}" in
    --foreground) FOREGROUND=true ;;
    --help|-h) usage; exit 0 ;;
    *) printf 'ERROR: unknown argument: %s\n' "${arg}" >&2; exit 1 ;;
  esac
done

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || {
    printf 'ERROR: missing required command: %s\n' "${command_name}" >&2
    exit 1
  }
}

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

create_token_file() {
  local token_file="$1"
  if [[ -f "${token_file}" ]]; then
    return
  fi
  umask 077
  mkdir -p "$(dirname "${token_file}")"
  if command -v openssl >/dev/null 2>&1; then
    openssl rand -hex 32 > "${token_file}"
  else
    python3 - <<'PY' > "${token_file}"
import secrets
print(secrets.token_hex(32))
PY
  fi
}

require_command cargo
require_command python3

DATA_ROOT="$(resolve_data_root)"
OPS_DIR="${DATA_ROOT}/ops"
TOKEN_FILE="${IGY6_HOST_BRIDGE_TOKEN_FILE:-${OPS_DIR}/host-bridge.token}"
PID_FILE="${OPS_DIR}/host-bridge.pid"
LOG_FILE="${OPS_DIR}/host-bridge.log"

mkdir -p "${OPS_DIR}"
create_token_file "${TOKEN_FILE}"

if [[ -f "${PID_FILE}" ]]; then
  EXISTING_PID="$(cat "${PID_FILE}")"
  if [[ -n "${EXISTING_PID}" ]] && kill -0 "${EXISTING_PID}" >/dev/null 2>&1; then
    printf 'Host bridge already appears to be running with PID %s.\n' "${EXISTING_PID}"
    exit 0
  fi
fi

CMD=(
  cargo run
  --manifest-path "${REPO_ROOT}/crates/igy6-host-bridge/Cargo.toml"
  --
  --host "${HOST}"
  --port "${PORT}"
  --repo-root "${REPO_ROOT}"
  --token-file "${TOKEN_FILE}"
)

printf 'Starting IGY6 host bridge on %s:%s\n' "${HOST}" "${PORT}"
printf 'Token file: %s\n' "${TOKEN_FILE}"

if [[ "${FOREGROUND}" == "true" ]]; then
  exec "${CMD[@]}"
fi

nohup "${CMD[@]}" > "${LOG_FILE}" 2>&1 &
PID="$!"
printf '%s\n' "${PID}" > "${PID_FILE}"
printf 'Host bridge started with PID %s. Log: %s\n' "${PID}" "${LOG_FILE}"
