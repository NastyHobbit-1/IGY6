#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
ENV_EXAMPLE="${REPO_ROOT}/.env.example"
MODE="check"
FAILURES=0

usage() {
  cat <<'EOF'
IGY6 fresh-clone startup validation

Usage:
  scripts/fresh-clone-startup-check.sh [--check] [--help]

Default:
  --check  Validate that a clean checkout has the tracked files, example
           configuration, Compose config, Rust worker command surface, and
           post-cutover smoke checks needed before live startup.

Safety:
  - Does not create, read, or mutate .env.
  - Does not create, read, or mutate IGY6_DATA_ROOT.
  - Does not start or stop Docker Compose services.
  - Does not run worker daemon mode or broad worker queues.
  - Does not install dependencies, pull images, or clone external repositories.
EOF
}

check() {
  printf 'CHECK %s\n' "$1"
}

pass() {
  printf 'PASS  %s\n' "$1"
}

fail() {
  printf 'FAIL  %s\n' "$1" >&2
  FAILURES=$((FAILURES + 1))
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

assert_file_exists() {
  local path="$1"
  [[ -f "${REPO_ROOT}/${path}" ]]
}

assert_required_files_exist() {
  local required_files=(
    "AGENTS.md"
    "Cargo.toml"
    "Cargo.lock"
    "README.md"
    ".env.example"
    "infra/docker-compose.yml"
    "crates/igy6-gateway/Dockerfile"
    "crates/igy6-worker/Dockerfile"
    "apps/web/package.json"
    "configs/rust-cutover-manifest.json"
    "configs/legacy-fastapi-route-classification.json"
    "scripts/post-cutover-runtime-audit.py"
    "scripts/post-cutover-smoke.sh"
    "scripts/rust-cutover.sh"
    "scripts/rust-route-parity.py"
  )

  local path
  for path in "${required_files[@]}"; do
    if ! assert_file_exists "${path}"; then
      printf 'missing required fresh-clone file: %s\n' "${path}" >&2
      return 1
    fi
  done
}

assert_env_example_complete() {
  python3 - "${COMPOSE_FILE}" "${ENV_EXAMPLE}" <<'PY'
import re
import sys
from pathlib import Path

compose_path = Path(sys.argv[1])
env_path = Path(sys.argv[2])

compose = compose_path.read_text(encoding="utf-8")
env_text = env_path.read_text(encoding="utf-8")

env_keys = set()
for line in env_text.splitlines():
    line = line.strip()
    if not line or line.startswith("#") or "=" not in line:
        continue
    env_keys.add(line.split("=", 1)[0])

compose_vars = set(re.findall(r"\$\{([A-Za-z_][A-Za-z0-9_]*)\}", compose))
missing = sorted(compose_vars - env_keys)
if missing:
    raise SystemExit(f".env.example is missing Compose variables: {', '.join(missing)}")

required_runtime_keys = {
    "APP_PORT",
    "DATABASE_URL",
    "IGY6_DATA_ROOT",
    "QDRANT_URL",
    "QDRANT_CHUNK_COLLECTION",
    "QDRANT_CHUNK_VECTOR_SIZE",
}
missing_runtime = sorted(required_runtime_keys - env_keys)
if missing_runtime:
    raise SystemExit(f".env.example is missing runtime keys: {', '.join(missing_runtime)}")

igy6_data_root = None
for line in env_text.splitlines():
    if line.startswith("IGY6_DATA_ROOT="):
        igy6_data_root = line.split("=", 1)[1].strip()
        break

if not igy6_data_root:
    raise SystemExit("IGY6_DATA_ROOT is empty in .env.example")
if igy6_data_root in {".", "./", ""}:
    raise SystemExit("IGY6_DATA_ROOT must not point at the repository root")
PY
}

assert_compose_fresh_clone_posture() {
  local compose_config
  compose_config="$(mktemp -t igy6-fresh-clone-compose.XXXXXX)"

  docker compose -f "${COMPOSE_FILE}" --env-file "${ENV_EXAMPLE}" config >"${compose_config}"

  grep -Fq "dockerfile: crates/igy6-gateway/Dockerfile" "${compose_config}" \
    || { rm -f "${compose_config}"; return 1; }
  grep -Fq "dockerfile: crates/igy6-worker/Dockerfile" "${compose_config}" \
    || { rm -f "${compose_config}"; return 1; }

  if grep -Eq '(^|[[:space:]])legacy-api:' "${compose_config}"; then
    rm -f "${compose_config}"
    return 1
  fi
  if grep -Eq '(^|[[:space:]])beat:' "${compose_config}"; then
    rm -f "${compose_config}"
    return 1
  fi
  if grep -Fq "services/worker" "${compose_config}"; then
    rm -f "${compose_config}"
    return 1
  fi

  rm -f "${compose_config}"
}

assert_manifest_fresh_clone_posture() {
  python3 - "${REPO_ROOT}/configs/rust-cutover-manifest.json" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = json.load(handle)

final_audit = manifest.get("final_rust_only_runtime_audit", {})
fresh = manifest.get("fresh_clone_startup_validation", {})

checks = {
    "target_architecture": manifest.get("target_architecture") == "rust-only-application-runtime",
    "fastapi_fallback_required": manifest.get("fastapi_fallback_required") is False,
    "python_celery_worker_active": final_audit.get("python_celery_worker_active") is False,
    "python_celery_beat_active": final_audit.get("python_celery_beat_active") is False,
    "rust_only_application_runtime_claimed": final_audit.get("rust_only_application_runtime_claimed") is True,
    "fresh_clone_command": fresh.get("command") == "scripts/fresh-clone-startup-check.sh --check",
    "fresh_clone_non_destructive": fresh.get("default_non_destructive") is True,
}

failed = [name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit(f"manifest fresh-clone posture checks failed: {', '.join(failed)}")
PY
}

assert_worker_check_output() {
  local output
  output="$(cargo run -p igy6-worker -- --check)"
  WORKER_CHECK_OUTPUT="${output}" python3 - <<'PY'
import json
import os

payload = json.loads(os.environ["WORKER_CHECK_OUTPUT"])
checks = {
    "mode": payload.get("mode") == "check",
    "mutates_runtime_data": payload.get("mutates_runtime_data") is False,
    "python_celery_worker_required": payload.get("python_celery_worker_required") is False,
    "python_celery_beat_required": payload.get("python_celery_beat_required") is False,
    "rust_only_runtime_claimed": payload.get("rust_only_runtime_claimed") is True,
}
failed = [name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit(f"worker check output failed: {', '.join(failed)}")
PY
}

run_check() {
  cd "${REPO_ROOT}"

  check "tool availability"
  require_command bash
  require_command cargo
  require_command docker
  require_command git
  require_command npm
  require_command python3

  run_required "fresh-clone required files exist" assert_required_files_exist
  run_required ".env.example covers Compose and runtime keys" assert_env_example_complete
  run_required "manifest JSON is valid" assert_manifest_json_valid
  run_required "manifest records fresh-clone Rust-only posture" assert_manifest_fresh_clone_posture
  run_required "Docker Compose config is valid from .env.example" assert_compose_fresh_clone_posture
  run_required "post-cutover runtime audit passes" python3 "${REPO_ROOT}/scripts/post-cutover-runtime-audit.py"
  run_required "route parity guard passes" python3 "${REPO_ROOT}/scripts/rust-route-parity.py" --check
  run_required "Rust worker help renders" cargo run -p igy6-worker -- --help
  run_required "Rust worker check is non-mutating and Rust-only" assert_worker_check_output
  run_required "post-cutover smoke suite passes" "${REPO_ROOT}/scripts/post-cutover-smoke.sh" --check

  if [[ "${FAILURES}" -gt 0 ]]; then
    exit 1
  fi
}

assert_manifest_json_valid() {
  python3 -m json.tool "${REPO_ROOT}/configs/rust-cutover-manifest.json" >/dev/null
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --check)
      MODE="check"
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      fail "unknown option: $1"
      usage
      exit 2
      ;;
  esac
  shift
done

case "${MODE}" in
  check)
    run_check
    ;;
  *)
    fail "unknown mode: ${MODE}"
    usage
    exit 2
    ;;
esac
