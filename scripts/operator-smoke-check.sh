#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"
COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
API_URL="${IGY6_OPERATOR_API_URL:-http://127.0.0.1:8000}"
WEB_URL="${IGY6_OPERATOR_WEB_URL:-http://127.0.0.1:3000}"
MODE=""
FAILURES=0
STARTED_STACK=0
RUN_PID=""
RUN_LOG="${TMPDIR:-/tmp}/igy6-operator-smoke-run.log"
PAGE_FILE="${TMPDIR:-/tmp}/igy6-operator-page.html"
RETRIEVAL_FILE="${TMPDIR:-/tmp}/igy6-operator-retrieval.json"
PORTS=(3000 8000 8765)

usage() {
  cat <<'EOF'
Usage: scripts/operator-smoke-check.sh --check
       scripts/operator-smoke-check.sh --run
       scripts/operator-smoke-check.sh --help

Modes:
  --check  Verify prerequisites and configuration only. Does not start the
           stack and does not mutate runtime data.
  --run    Run the full local operator smoke using synthetic test data.
  --help   Show this help.

Safety:
  - Does not print .env contents or IGY6_DATA_ROOT values.
  - Does not dump runtime/private data from IGY6_DATA_ROOT.
  - Uses only synthetic smoke data.
  - Does not kill existing processes.
  - Stops the stack on exit if this script started it.
  - Does not remove volumes, images, files, or runtime data.
  - Does not change Docker permissions, user groups, or system services.
EOF
}

pass() {
  printf 'PASS %s\n' "$1"
}

fail() {
  printf 'FAIL %s\n' "$1" >&2
  FAILURES=$((FAILURES + 1))
}

info() {
  printf 'INFO %s\n' "$1"
}

finish_step() {
  if [[ "${FAILURES}" -gt 0 ]]; then
    exit 1
  fi
}

require_repo_root() {
  if [[ ! -f "${COMPOSE_FILE}" || ! -f "${REPO_ROOT}/scripts/run.sh" || ! -f "${REPO_ROOT}/scripts/stop.sh" ]]; then
    fail "required repo files are missing; run from the IGY6 repository root"
    finish_step
  fi
  pass "required repo files are present"
}

require_command() {
  local command_name="$1"
  if command -v "${command_name}" >/dev/null 2>&1; then
    pass "${command_name} is available"
  else
    fail "${command_name} is not available"
  fi
}

require_basic_commands() {
  require_command curl
  require_command npm
  require_command python3
  if command -v ss >/dev/null 2>&1 || command -v netstat >/dev/null 2>&1 || command -v lsof >/dev/null 2>&1; then
    pass "port inspection command is available"
  else
    fail "need ss, netstat, or lsof for port conflict checks"
  fi
}

print_docker_guidance() {
  info "Docker access manual checks:"
  info "  id"
  info "  ls -l /var/run/docker.sock"
  info "  docker ps"
  info "Likely permission fix, if Docker is installed and running: add the current user to the docker group, then restart the shell or WSL session."
  info "This script will not run that fix automatically."
}

require_docker_access() {
  local docker_output
  local docker_status

  if ! command -v docker >/dev/null 2>&1; then
    fail "docker command is missing"
    print_docker_guidance
    return
  fi
  pass "docker command is available"

  set +e
  docker_output="$(docker ps 2>&1)"
  docker_status=$?
  set -e

  if [[ "${docker_status}" -eq 0 ]]; then
    pass "Docker daemon is accessible"
    pass "current user can run docker commands"
  else
    if printf '%s' "${docker_output}" | grep -qiE 'permission denied|/var/run/docker\.sock'; then
      fail "permission denied connecting to /var/run/docker.sock"
      fail "current user cannot run docker commands"
    elif printf '%s' "${docker_output}" | grep -qiE 'cannot connect to the docker daemon|docker daemon|is the docker daemon running'; then
      fail "Docker daemon is unavailable"
      fail "current user cannot run docker commands"
    else
      fail "current user cannot run docker commands"
    fi
    print_docker_guidance
    return
  fi

  if docker compose version >/dev/null 2>&1; then
    pass "Docker Compose plugin is available"
  else
    fail "Docker Compose plugin is not available"
  fi
}

compose_config() {
  if docker compose -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" config --quiet >/dev/null; then
    pass "Docker Compose config is valid"
  else
    fail "Docker Compose config failed"
  fi
}

check_env_file() {
  if [[ -f "${ENV_FILE}" ]]; then
    pass ".env file exists"
  else
    fail ".env file missing; create it from .env.example before operator smoke"
    return
  fi

  if grep -qE '^[[:space:]]*(export[[:space:]]+)?IGY6_DATA_ROOT=' "${ENV_FILE}"; then
    pass "IGY6_DATA_ROOT key is present"
  else
    fail "IGY6_DATA_ROOT key is missing"
  fi
}

check_data_root_dir() {
  local result
  result="$(REPO_ROOT="${REPO_ROOT}" ENV_FILE="${ENV_FILE}" python3 - <<'PY'
import os
import sys

repo = os.environ["REPO_ROOT"]
env_file = os.environ["ENV_FILE"]
value = None
try:
    with open(env_file, encoding="utf-8") as handle:
        for raw in handle:
            line = raw.strip()
            if not line or line.startswith("#"):
                continue
            if line.startswith("export "):
                line = line[len("export "):].strip()
            if line.startswith("IGY6_DATA_ROOT="):
                value = line.split("=", 1)[1].split("#", 1)[0].strip()
except OSError:
    print("missing")
    sys.exit(0)

if value is None or value == "":
    print("missing")
    sys.exit(0)
if (value.startswith('"') and value.endswith('"')) or (value.startswith("'") and value.endswith("'")):
    value = value[1:-1]
path = value if os.path.isabs(value) else os.path.abspath(os.path.join(repo, value))
print("exists" if os.path.isdir(path) else "missing")
PY
)"
  if [[ "${result}" == "exists" ]]; then
    pass "IGY6_DATA_ROOT directory exists"
  else
    fail "IGY6_DATA_ROOT directory is missing"
  fi
}

port_listening() {
  local port="$1"
  if command -v ss >/dev/null 2>&1; then
    ss -ltn "( sport = :${port} )" 2>/dev/null | grep -q ":${port}"
  elif command -v netstat >/dev/null 2>&1; then
    netstat -ltn 2>/dev/null | grep -Eq "[:.]${port}[[:space:]]"
  elif command -v lsof >/dev/null 2>&1; then
    lsof -iTCP:"${port}" -sTCP:LISTEN >/dev/null 2>&1
  else
    return 1
  fi
}

check_ports_clear() {
  local label="$1"
  local port
  for port in "${PORTS[@]}"; do
    if port_listening "${port}"; then
      fail "${label}: port ${port} is already listening; no process was killed"
    else
      pass "${label}: port ${port} is clear"
    fi
  done
}

http_status() {
  local url="$1"
  curl --silent --show-error --output /dev/null --write-out '%{http_code}' --max-time 10 "${url}" 2>/dev/null || true
}

probe_http_200() {
  local label="$1"
  local url="$2"
  local status
  status="$(http_status "${url}")"
  if [[ "${status}" == "200" ]]; then
    pass "${label} returned HTTP 200"
  else
    fail "${label} returned HTTP ${status:-none}"
  fi
}

wait_for_http_200() {
  local label="$1"
  local url="$2"
  local attempts="${3:-60}"
  local delay="${4:-2}"
  local attempt status
  for ((attempt = 1; attempt <= attempts; attempt++)); do
    if [[ -n "${RUN_PID}" ]] && ! kill -0 "${RUN_PID}" 2>/dev/null; then
      fail "scripts/run.sh exited before ${label} became ready; see ${RUN_LOG}"
      return 1
    fi
    status="$(http_status "${url}")"
    if [[ "${status}" == "200" ]]; then
      pass "${label} returned HTTP 200"
      return 0
    fi
    sleep "${delay}"
  done
  fail "${label} did not return HTTP 200; last status ${status:-none}"
  return 1
}

fetch_web_page() {
  local status
  status="$(curl --silent --show-error --output "${PAGE_FILE}" --write-out '%{http_code}' --max-time 10 "${WEB_URL}/" 2>/dev/null || true)"
  if [[ "${status}" == "200" ]]; then
    pass "web UI page fetched for marker checks"
  else
    fail "web UI page fetch returned HTTP ${status:-none}"
  fi
}

check_marker() {
  local label="$1"
  local marker="$2"
  if grep -q "${marker}" "${PAGE_FILE}" 2>/dev/null; then
    pass "${label} marker present"
  else
    fail "${label} marker missing"
  fi
}

check_retrieval_preview() {
  local status
  status="$(curl --silent --show-error --request POST "${API_URL%/}/chat/retrieval-preview" \
    --header 'Content-Type: application/json' \
    --data '{"message":"Find blue-raven-117 in my uploaded evidence.","limit":5}' \
    --output "${RETRIEVAL_FILE}" \
    --write-out '%{http_code}' \
    --max-time 15 2>/dev/null || true)"
  if [[ "${status}" != "200" ]]; then
    fail "retrieval preview returned HTTP ${status:-none}"
    return
  fi
  pass "retrieval preview returned HTTP 200"
  if python3 - "${RETRIEVAL_FILE}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)
context = payload.get("retrieval_context") or {}
items = context.get("hits") or payload.get("items") or []
status = payload.get("answer_status") or payload.get("status") or "ok"
print(f"PASS retrieval preview summary: items={len(items)} status={status}")
if not items:
    raise SystemExit(1)
PY
  then
    :
  else
    fail "retrieval preview returned no hits"
  fi
}

run_web_build() {
  if npm --prefix "${REPO_ROOT}/apps/web" run build; then
    pass "web build passed"
  else
    fail "web build failed"
  fi
}

start_stack() {
  info "starting stack with scripts/run.sh; log: ${RUN_LOG}"
  : >"${RUN_LOG}"
  "${REPO_ROOT}/scripts/run.sh" >"${RUN_LOG}" 2>&1 &
  RUN_PID="$!"
  STARTED_STACK=1
  pass "scripts/run.sh started in background"
}

stop_started_stack() {
  if [[ "${STARTED_STACK}" -eq 1 ]]; then
    info "stopping stack with scripts/stop.sh"
    if "${REPO_ROOT}/scripts/stop.sh"; then
      pass "stack stop completed"
    else
      fail "stack stop failed"
    fi
    if [[ -n "${RUN_PID}" ]] && kill -0 "${RUN_PID}" 2>/dev/null; then
      wait "${RUN_PID}" 2>/dev/null || true
    fi
    STARTED_STACK=0
  fi
}

cleanup() {
  stop_started_stack
}

run_check_mode() {
  cd "${REPO_ROOT}"
  require_repo_root
  require_basic_commands
  check_env_file
  check_data_root_dir
  require_docker_access
  finish_step
  compose_config
  check_ports_clear "preflight"
  finish_step
}

run_full_mode() {
  cd "${REPO_ROOT}"
  require_repo_root
  require_basic_commands
  check_env_file
  check_data_root_dir
  require_docker_access
  finish_step
  compose_config
  run_web_build
  check_ports_clear "before start"
  finish_step

  trap cleanup EXIT
  start_stack
  wait_for_http_200 "API live" "${API_URL%/}/health/live"
  wait_for_http_200 "API ready" "${API_URL%/}/health/ready"
  wait_for_http_200 "web UI" "${WEB_URL%/}/"
  finish_step

  if python3 "${REPO_ROOT}/scripts/e2e-manual-upload-smoke.py" --run --api-base-url "${API_URL}" --web-url "${WEB_URL}"; then
    pass "synthetic manual upload smoke passed"
  else
    fail "synthetic manual upload smoke failed"
  fi
  fetch_web_page
  check_marker "guided manual upload result" "data-guided-manual-result"
  check_marker "work item status" "data-work-status-item"
  check_marker "retrieval results UI" "data-chat-preview-results"
  check_marker "report workflow" "data-basic-report-workflow"
  check_marker "feedback/outcome workflow" "data-evidence-feedback-workflow"
  check_marker "source/evidence history" "data-source-evidence-history"
  check_retrieval_preview
  stop_started_stack
  check_ports_clear "after stop"
  finish_step
}

if [[ "$#" -ne 1 ]]; then
  usage
  exit 2
fi

case "$1" in
  --help|-h)
    usage
    exit 0
    ;;
  --check)
    MODE="check"
    ;;
  --run)
    MODE="run"
    ;;
  *)
    usage
    exit 2
    ;;
esac

case "${MODE}" in
  check)
    run_check_mode
    ;;
  run)
    run_full_mode
    ;;
esac
