#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
ENV_FILE="${REPO_ROOT}/.env.example"
API_URL="${POST_CUTOVER_SMOKE_API_URL:-http://127.0.0.1:8000}"
MODE="check"
REQUIRE_RUNNING=0
FAILURES=0

usage() {
  cat <<'EOF'
IGY6 post-cutover Rust-only runtime smoke suite

Usage:
  scripts/post-cutover-smoke.sh [--check] [--require-running] [--api-url URL] [--help]

Default:
  --check   Run non-destructive static and command-level runtime checks.

Options:
  --require-running  Fail if the local Rust API is not reachable.
  --api-url URL      API base URL for optional live health probes.
  --help             Show this help.

Environment:
  POST_CUTOVER_SMOKE_API_URL  Optional API base URL. Defaults to http://127.0.0.1:8000.

Safety:
  - Does not read or mutate .env.
  - Does not start or stop Docker Compose services.
  - Does not run worker daemon mode or broad worker queues.
  - Does not touch runtime/private data or IGY6_DATA_ROOT contents.
EOF
}

check() {
  printf 'CHECK %s\n' "$1"
}

pass() {
  printf 'PASS  %s\n' "$1"
}

skip() {
  printf 'SKIP  %s\n' "$1"
}

fail() {
  printf 'FAIL  %s\n' "$1" >&2
  FAILURES=$((FAILURES + 1))
}

die() {
  printf 'ERROR %s\n' "$*" >&2
  exit 2
}

require_command() {
  local command_name="$1"
  if command -v "${command_name}" >/dev/null 2>&1; then
    pass "${command_name} is available"
  else
    fail "${command_name} is not available"
  fi
}

run_required() {
  local label="$1"
  shift
  check "${label}"
  if "$@"; then
    pass "${label}"
  else
    fail "${label}"
  fi
}

assert_manifest() {
  python3 - "${REPO_ROOT}/configs/rust-cutover-manifest.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = json.load(handle)

final_audit = manifest.get("final_rust_only_runtime_audit", {})
smoke = manifest.get("post_cutover_runtime_smoke_suite", {})

checks = {
    "target_architecture": manifest.get("target_architecture") == "rust-only-application-runtime",
    "fastapi_fallback_required": manifest.get("fastapi_fallback_required") is False,
    "python_celery_worker_active": final_audit.get("python_celery_worker_active") is False,
    "python_celery_beat_active": final_audit.get("python_celery_beat_active") is False,
    "rust_only_application_runtime_claimed": final_audit.get("rust_only_application_runtime_claimed") is True,
    "smoke_command": smoke.get("command") == "scripts/post-cutover-smoke.sh --check",
    "smoke_non_destructive": smoke.get("default_non_destructive") is True,
}

failed = [name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit(f"manifest smoke/runtime posture checks failed: {', '.join(failed)}")
PY
}

assert_manifest_json_valid() {
  python3 -m json.tool "${REPO_ROOT}/configs/rust-cutover-manifest.json" >/dev/null
}

assert_compose_posture() {
  local compose_config
  compose_config="$(mktemp -t igy6-post-cutover-compose.XXXXXX)"

  docker compose -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" config >"${compose_config}"

  grep -Fq "dockerfile: crates/igy6-gateway/Dockerfile" "${compose_config}" \
    || die "Compose API service is not built from crates/igy6-gateway/Dockerfile"
  grep -Fq "dockerfile: crates/igy6-worker/Dockerfile" "${compose_config}" \
    || die "Compose worker service is not built from crates/igy6-worker/Dockerfile"

  if grep -Eq '(^|[[:space:]])legacy-api:' "${compose_config}"; then
    die "Compose config still contains a legacy-api service"
  fi
  if grep -Eq '(^|[[:space:]])beat:' "${compose_config}"; then
    die "Compose config still contains a beat service"
  fi
  if grep -Fq "services/worker" "${compose_config}"; then
    rm -f "${compose_config}"
    die "Compose config still references services/worker"
  fi
  rm -f "${compose_config}"
}

assert_worker_check_output() {
  local output
  output="$(cargo run -p igy6-worker -- --check)"
  WORKER_CHECK_OUTPUT="${output}" python3 - <<'PY'
import json
import os
import sys

payload = json.loads(os.environ["WORKER_CHECK_OUTPUT"])
checks = {
    "mode": payload.get("mode") == "check",
    "diff": payload.get("diff") == "DIFF-165",
    "python_celery_worker_required": payload.get("python_celery_worker_required") is False,
    "python_celery_beat_required": payload.get("python_celery_beat_required") is False,
    "rust_only_runtime_claimed": payload.get("rust_only_runtime_claimed") is True,
}
failed = [name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit(f"worker check output failed: {', '.join(failed)}")
PY
}

probe_api_endpoint() {
  local label="$1"
  local path="$2"
  local url="${API_URL%/}${path}"
  local status

  if ! command -v curl >/dev/null 2>&1; then
    if [[ "${REQUIRE_RUNNING}" -eq 1 ]]; then
      fail "curl is required for live API probe: ${label}"
    else
      skip "curl unavailable; live API probe skipped: ${label}"
    fi
    return
  fi

  status="$(curl --silent --show-error --output /dev/null --write-out '%{http_code}' --max-time 3 "${url}" 2>/dev/null || true)"
  case "${status}" in
    2*|3*)
      pass "${label} responded with HTTP ${status}"
      ;;
    000|"")
      if [[ "${REQUIRE_RUNNING}" -eq 1 ]]; then
        fail "${label} did not respond at ${url}"
      else
        skip "${label} not reachable at ${url}; use --require-running to make this fatal"
      fi
      ;;
    *)
      fail "${label} responded with HTTP ${status} at ${url}"
      ;;
  esac
}

run_check() {
  cd "${REPO_ROOT}"

  check "tool availability"
  require_command python3
  require_command cargo
  require_command docker

  run_required "manifest JSON is valid" assert_manifest_json_valid
  run_required "manifest claims Rust-only application API/worker runtime" assert_manifest
  run_required "post-cutover audit passes" python3 "${REPO_ROOT}/scripts/post-cutover-runtime-audit.py"
  run_required "route parity guard passes" python3 "${REPO_ROOT}/scripts/rust-route-parity.py" --check
  run_required "Rust cutover check passes" "${REPO_ROOT}/scripts/rust-cutover.sh" --check
  run_required "Docker Compose post-cutover posture is valid" assert_compose_posture
  run_required "Rust worker help renders" cargo run -p igy6-worker -- --help
  run_required "Rust worker check reports Rust-only posture" assert_worker_check_output

  check "optional live Rust API probes"
  probe_api_endpoint "API live" "/health/live"
  probe_api_endpoint "API ready" "/health/ready"
  probe_api_endpoint "Rust migration status" "/rust-migration/status"

  if [[ "${FAILURES}" -gt 0 ]]; then
    exit 1
  fi
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --check)
      MODE="check"
      ;;
    --require-running)
      REQUIRE_RUNNING=1
      ;;
    --api-url)
      shift
      [[ "$#" -gt 0 ]] || die "--api-url requires a URL"
      API_URL="$1"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "Unknown argument: $1"
      ;;
  esac
  shift
done

case "${MODE}" in
  check)
    run_check
    ;;
  *)
    die "Unknown mode: ${MODE}"
    ;;
esac
