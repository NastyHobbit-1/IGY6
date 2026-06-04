#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"
COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
API_URL="${IGY6_OPERATOR_API_URL:-http://127.0.0.1:8000}"
WEB_URL="${IGY6_OPERATOR_WEB_URL:-http://127.0.0.1:3000}"
MODE=""
RECORD_RESULTS=0
OVERALL_STATUS="unknown"
FAILURE_REASON=""
FAILURES=0
STARTED_STACK=0
STACK_STARTED_BY_SCRIPT=0
STACK_STOPPED_BY_SCRIPT=0
RUN_PID=""
RUN_LOG="${TMPDIR:-/tmp}/igy6-operator-smoke-run.log"
PAGE_FILE="${TMPDIR:-/tmp}/igy6-operator-page.html"
RETRIEVAL_FILE="${TMPDIR:-/tmp}/igy6-operator-retrieval.json"
E2E_OUTPUT_FILE="${TMPDIR:-/tmp}/igy6-operator-e2e-output.log"
RESULT_DIR="${REPO_ROOT}/.igy6-local/smoke-results"
RESULT_FILE=""
PORTS=(3000 8000 8765)
STEP_NAMES=()
STEP_STATUSES=()
STEP_MESSAGES=()
API_LIVE_STATUS=""
API_READY_STATUS=""
WEB_STATUS=""
RETRIEVAL_STATUS=""
RETRIEVAL_ITEMS_COUNT=""
RETRIEVAL_ANSWER_STATUS=""
ARTIFACTS_COUNT=""
DOCUMENTS_COUNT=""
CHUNKS_COUNT=""
EVIDENCE_ITEMS_COUNT=""
IGY6_DATA_ROOT_PRESENT="false"

usage() {
  cat <<'EOF'
Usage: scripts/operator-smoke-check.sh --check
       scripts/operator-smoke-check.sh --run
       scripts/operator-smoke-check.sh --run --record
       scripts/operator-smoke-check.sh --run-record
       scripts/operator-smoke-check.sh --help

Modes:
  --check  Verify prerequisites and configuration only. Does not start the
           stack and does not mutate runtime data.
  --run    Run the full local operator smoke using synthetic test data.
  --record With --run, write a safe JSON result summary under
           .igy6-local/smoke-results/.
  --run-record
           Equivalent to --run --record.
  --help   Show this help.

Safety:
  - Does not print .env contents or IGY6_DATA_ROOT values.
  - Does not dump runtime/private data from IGY6_DATA_ROOT.
  - Uses only synthetic smoke data.
  - Does not kill existing processes.
  - Stops the stack on exit if this script started it.
  - Does not remove volumes, images, files, or runtime data.
  - Does not change Docker permissions, user groups, or system services.
  - Result recording is optional and stores only safe summaries, never .env
    values, raw uploaded text, runtime/private data, or full logs.
EOF
}

record_step() {
  STEP_NAMES+=("$1")
  STEP_STATUSES+=("$2")
  STEP_MESSAGES+=("$3")
}

pass() {
  printf 'PASS %s\n' "$1"
  record_step "$1" "pass" "$1"
}

fail() {
  printf 'FAIL %s\n' "$1" >&2
  record_step "$1" "fail" "$1"
  if [[ -z "${FAILURE_REASON}" ]]; then
    FAILURE_REASON="$1"
  fi
  FAILURES=$((FAILURES + 1))
}

info() {
  printf 'INFO %s\n' "$1"
}

finish_step() {
  if [[ "${FAILURES}" -gt 0 ]]; then
    OVERALL_STATUS="failed"
    exit 1
  fi
}

smoke_token_hash() {
  python3 - <<'PY'
import hashlib
print(hashlib.sha256(b"blue-raven-117").hexdigest())
PY
}

write_result_record() {
  local exit_status="$1"
  local created_at
  local timestamp
  local repo_branch
  local repo_head
  local steps_file

  if [[ "${RECORD_RESULTS}" -ne 1 || "${MODE}" != "run" ]]; then
    return
  fi

  if [[ "${exit_status}" -eq 0 && "${FAILURES}" -eq 0 ]]; then
    OVERALL_STATUS="passed"
  else
    OVERALL_STATUS="failed"
    if [[ -z "${FAILURE_REASON}" ]]; then
      FAILURE_REASON="operator smoke exited with status ${exit_status}"
    fi
  fi

  created_at="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
  timestamp="$(date -u '+%Y%m%dT%H%M%SZ')"
  repo_branch="$(git -C "${REPO_ROOT}" branch --show-current 2>/dev/null || true)"
  repo_head="$(git -C "${REPO_ROOT}" rev-parse --short HEAD 2>/dev/null || true)"
  RESULT_FILE="${RESULT_DIR}/operator-smoke-${timestamp}.json"
  steps_file="${TMPDIR:-/tmp}/igy6-operator-smoke-steps-${timestamp}.$$"

  mkdir -p "${RESULT_DIR}"
  : >"${steps_file}"
  local i
  for ((i = 0; i < ${#STEP_NAMES[@]}; i++)); do
    printf '%s\t%s\t%s\n' "${STEP_NAMES[$i]}" "${STEP_STATUSES[$i]}" "${STEP_MESSAGES[$i]}" >>"${steps_file}"
  done

  CREATED_AT="${created_at}" \
  REPO_BRANCH="${repo_branch}" \
  REPO_HEAD="${repo_head}" \
  RESULT_FILE="${RESULT_FILE}" \
  STEPS_FILE="${steps_file}" \
  OVERALL_STATUS="${OVERALL_STATUS}" \
  FAILURE_REASON="${FAILURE_REASON}" \
  API_LIVE_STATUS="${API_LIVE_STATUS}" \
  API_READY_STATUS="${API_READY_STATUS}" \
  WEB_STATUS="${WEB_STATUS}" \
  RETRIEVAL_STATUS="${RETRIEVAL_STATUS}" \
  RETRIEVAL_ITEMS_COUNT="${RETRIEVAL_ITEMS_COUNT}" \
  RETRIEVAL_ANSWER_STATUS="${RETRIEVAL_ANSWER_STATUS}" \
  ARTIFACTS_COUNT="${ARTIFACTS_COUNT}" \
  DOCUMENTS_COUNT="${DOCUMENTS_COUNT}" \
  CHUNKS_COUNT="${CHUNKS_COUNT}" \
  EVIDENCE_ITEMS_COUNT="${EVIDENCE_ITEMS_COUNT}" \
  IGY6_DATA_ROOT_PRESENT="${IGY6_DATA_ROOT_PRESENT}" \
  STACK_STARTED_BY_SCRIPT="${STACK_STARTED_BY_SCRIPT}" \
  STACK_STOPPED_BY_SCRIPT="${STACK_STOPPED_BY_SCRIPT}" \
  SMOKE_TOKEN_HASH="$(smoke_token_hash)" \
  python3 - <<'PY'
import json
import os

def env_bool(name: str) -> bool:
    return os.environ.get(name, "").lower() in {"1", "true", "yes"}

def env_int_or_none(name: str):
    value = os.environ.get(name, "")
    if value == "":
        return None
    try:
        return int(value)
    except ValueError:
        return None

steps = []
with open(os.environ["STEPS_FILE"], encoding="utf-8") as handle:
    for raw in handle:
        name, status, message = raw.rstrip("\n").split("\t", 2)
        steps.append({"name": name, "status": status, "message": message})

payload = {
    "schema_version": 1,
    "created_at_utc": os.environ["CREATED_AT"],
    "repo_branch": os.environ["REPO_BRANCH"],
    "repo_head": os.environ["REPO_HEAD"],
    "smoke_script": "scripts/operator-smoke-check.sh",
    "mode": "run-record",
    "overall_status": os.environ["OVERALL_STATUS"],
    "steps": steps,
    "synthetic_token": {
        "marker": "redacted_synthetic_token_hash",
        "sha256": os.environ["SMOKE_TOKEN_HASH"],
    },
    "api_status": {
        "live_http_status": os.environ.get("API_LIVE_STATUS") or None,
        "ready_http_status": os.environ.get("API_READY_STATUS") or None,
        "retrieval_preview_http_status": os.environ.get("RETRIEVAL_STATUS") or None,
    },
    "web_status": {
        "root_http_status": os.environ.get("WEB_STATUS") or None,
    },
    "counts": {
        "artifacts": env_int_or_none("ARTIFACTS_COUNT"),
        "documents": env_int_or_none("DOCUMENTS_COUNT"),
        "chunks": env_int_or_none("CHUNKS_COUNT"),
        "evidence_items": env_int_or_none("EVIDENCE_ITEMS_COUNT"),
        "retrieval_items": env_int_or_none("RETRIEVAL_ITEMS_COUNT"),
    },
    "retrieval_summary": {
        "answer_status": os.environ.get("RETRIEVAL_ANSWER_STATUS") or None,
    },
    "ports_checked": [3000, 8000, 8765],
    "igy6_data_root_present": env_bool("IGY6_DATA_ROOT_PRESENT"),
    "stack_started_by_script": env_bool("STACK_STARTED_BY_SCRIPT"),
    "stack_stopped_by_script": env_bool("STACK_STOPPED_BY_SCRIPT"),
    "failure_reason": os.environ.get("FAILURE_REASON") or None,
}

with open(os.environ["RESULT_FILE"], "w", encoding="utf-8") as handle:
    json.dump(payload, handle, indent=2, sort_keys=True)
    handle.write("\n")
PY
  rm -f "${steps_file}"
  info "wrote smoke result summary ${RESULT_FILE}"
}

on_exit() {
  local exit_status="$?"
  cleanup
  write_result_record "${exit_status}"
  exit "${exit_status}"
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
    IGY6_DATA_ROOT_PRESENT="true"
    pass "IGY6_DATA_ROOT directory exists"
  else
    IGY6_DATA_ROOT_PRESENT="false"
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
  case "${label}" in
    "API live") API_LIVE_STATUS="${status}" ;;
    "API ready") API_READY_STATUS="${status}" ;;
    "web UI") WEB_STATUS="${status}" ;;
  esac
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
    case "${label}" in
      "API live") API_LIVE_STATUS="${status}" ;;
      "API ready") API_READY_STATUS="${status}" ;;
      "web UI") WEB_STATUS="${status}" ;;
    esac
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
  WEB_STATUS="${status}"
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
    RETRIEVAL_STATUS="${status}"
    fail "retrieval preview returned HTTP ${status:-none}"
    return
  fi
  RETRIEVAL_STATUS="${status}"
  pass "retrieval preview returned HTTP 200"
  if result="$(python3 - "${RETRIEVAL_FILE}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    payload = json.load(handle)
context = payload.get("retrieval_context") or {}
items = context.get("hits") or payload.get("items") or []
status = payload.get("answer_status") or payload.get("status") or "ok"
print(f"{len(items)}\t{status}")
if not items:
    raise SystemExit(1)
PY
  )"; then
    RETRIEVAL_ITEMS_COUNT="${result%%$'\t'*}"
    RETRIEVAL_ANSWER_STATUS="${result#*$'\t'}"
    pass "retrieval preview summary: items=${RETRIEVAL_ITEMS_COUNT} status=${RETRIEVAL_ANSWER_STATUS}"
  else
    fail "retrieval preview returned no hits"
  fi
}

record_e2e_counts() {
  if [[ ! -f "${E2E_OUTPUT_FILE}" ]]; then
    return
  fi
  while IFS= read -r line; do
    case "${line}" in
      PASS\ artifacts\ endpoint\ returned\ *\ records)
        ARTIFACTS_COUNT="$(printf '%s\n' "${line}" | sed -E 's/^PASS artifacts endpoint returned ([0-9]+) records$/\1/')"
        ;;
      PASS\ documents\ endpoint\ returned\ *\ records)
        DOCUMENTS_COUNT="$(printf '%s\n' "${line}" | sed -E 's/^PASS documents endpoint returned ([0-9]+) records$/\1/')"
        ;;
      PASS\ chunks\ endpoint\ returned\ *\ records)
        CHUNKS_COUNT="$(printf '%s\n' "${line}" | sed -E 's/^PASS chunks endpoint returned ([0-9]+) records$/\1/')"
        ;;
      PASS\ evidence\ items\ endpoint\ returned\ *\ records)
        EVIDENCE_ITEMS_COUNT="$(printf '%s\n' "${line}" | sed -E 's/^PASS evidence items endpoint returned ([0-9]+) records$/\1/')"
        ;;
    esac
  done <"${E2E_OUTPUT_FILE}"
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
  STACK_STARTED_BY_SCRIPT=1
  pass "scripts/run.sh started in background"
}

stop_started_stack() {
  if [[ "${STARTED_STACK}" -eq 1 ]]; then
    info "stopping stack with scripts/stop.sh"
    if "${REPO_ROOT}/scripts/stop.sh"; then
      STACK_STOPPED_BY_SCRIPT=1
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

  start_stack
  wait_for_http_200 "API live" "${API_URL%/}/health/live"
  wait_for_http_200 "API ready" "${API_URL%/}/health/ready"
  wait_for_http_200 "web UI" "${WEB_URL%/}/"
  finish_step

  : >"${E2E_OUTPUT_FILE}"
  if python3 "${REPO_ROOT}/scripts/e2e-manual-upload-smoke.py" --run --api-base-url "${API_URL}" --web-url "${WEB_URL}" 2>&1 | tee "${E2E_OUTPUT_FILE}"; then
    pass "synthetic manual upload smoke passed"
  else
    fail "synthetic manual upload smoke failed"
  fi
  record_e2e_counts
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

if [[ "$#" -lt 1 || "$#" -gt 2 ]]; then
  usage
  exit 2
fi

case "$1" in
  --help|-h)
    if [[ "$#" -ne 1 ]]; then
      usage
      exit 2
    fi
    usage
    exit 0
    ;;
  --check)
    if [[ "$#" -ne 1 ]]; then
      usage
      exit 2
    fi
    MODE="check"
    ;;
  --run)
    if [[ "$#" -eq 2 ]]; then
      if [[ "$2" == "--record" ]]; then
        RECORD_RESULTS=1
      else
        usage
        exit 2
      fi
    fi
    MODE="run"
    ;;
  --run-record)
    if [[ "$#" -ne 1 ]]; then
      usage
      exit 2
    fi
    MODE="run"
    RECORD_RESULTS=1
    ;;
  *)
    usage
    exit 2
    ;;
esac

trap on_exit EXIT

case "${MODE}" in
  check)
    run_check_mode
    ;;
  run)
    run_full_mode
    ;;
esac
