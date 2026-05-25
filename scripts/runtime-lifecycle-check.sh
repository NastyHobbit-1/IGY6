#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
ENV_EXAMPLE="${REPO_ROOT}/.env.example"
MANIFEST="${REPO_ROOT}/configs/rust-cutover-manifest.json"
MODE="check"
FAILURES=0

EXPECTED_SERVICES=(
  postgres
  redis
  qdrant
  neo4j
  mlflow
  phoenix
  api
  worker
  web
)

usage() {
  cat <<'EOF'
IGY6 runtime lifecycle validation

Usage:
  scripts/runtime-lifecycle-check.sh [--check] [--help]

Default:
  --check  Validate startup, shutdown, and restart command shapes plus
           post-cutover Compose ownership without starting or stopping services.

Planned lifecycle commands:
  start:    docker compose -f infra/docker-compose.yml --env-file .env up --build
  shutdown: docker compose -f infra/docker-compose.yml --env-file .env down
  restart:  docker compose -f infra/docker-compose.yml --env-file .env down
            docker compose -f infra/docker-compose.yml --env-file .env up --build -d

Safety:
  - Does not read or mutate .env.
  - Does not create, read, or mutate IGY6_DATA_ROOT.
  - Does not start, stop, restart, or remove Docker Compose services.
  - Does not run worker daemon mode or broad worker queues.
  - Does not remove volumes, images, archives, or runtime/private data.
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

assert_compose_lifecycle_posture() {
  local compose_config
  local compose_services
  compose_config="$(mktemp -t igy6-lifecycle-compose.XXXXXX)"
  compose_services="$(mktemp -t igy6-lifecycle-services.XXXXXX)"

  docker compose -f "${COMPOSE_FILE}" --env-file "${ENV_EXAMPLE}" config >"${compose_config}"
  docker compose -f "${COMPOSE_FILE}" --env-file "${ENV_EXAMPLE}" config --services >"${compose_services}"

  local service
  for service in "${EXPECTED_SERVICES[@]}"; do
    grep -Fxq "${service}" "${compose_services}" || {
      rm -f "${compose_config}" "${compose_services}"
      printf 'missing expected Compose service: %s\n' "${service}" >&2
      return 1
    }
  done

  if grep -Fxq "legacy-api" "${compose_services}"; then
    rm -f "${compose_config}" "${compose_services}"
    printf 'legacy-api service must not be active\n' >&2
    return 1
  fi
  if grep -Fxq "beat" "${compose_services}"; then
    rm -f "${compose_config}" "${compose_services}"
    printf 'beat service must not be active\n' >&2
    return 1
  fi

  grep -Fq "dockerfile: crates/igy6-gateway/Dockerfile" "${compose_config}" || {
    rm -f "${compose_config}" "${compose_services}"
    printf 'api service is not built from crates/igy6-gateway/Dockerfile\n' >&2
    return 1
  }
  grep -Fq "dockerfile: crates/igy6-worker/Dockerfile" "${compose_config}" || {
    rm -f "${compose_config}" "${compose_services}"
    printf 'worker service is not built from crates/igy6-worker/Dockerfile\n' >&2
    return 1
  }
  grep -Fq -- "- --daemon" "${compose_config}" || {
    rm -f "${compose_config}" "${compose_services}"
    printf 'worker service command does not include --daemon\n' >&2
    return 1
  }
  grep -Fq -- "- --claim-limit" "${compose_config}" || {
    rm -f "${compose_config}" "${compose_services}"
    printf 'worker service command does not include --claim-limit\n' >&2
    return 1
  }
  grep -Fq -- "- --poll-interval-ms" "${compose_config}" || {
    rm -f "${compose_config}" "${compose_services}"
    printf 'worker service command does not include --poll-interval-ms\n' >&2
    return 1
  }
  if grep -Fq "services/worker" "${compose_config}"; then
    rm -f "${compose_config}" "${compose_services}"
    printf 'Compose config must not reference services/worker\n' >&2
    return 1
  fi

  rm -f "${compose_config}" "${compose_services}"
}

assert_lifecycle_command_shapes() {
  python3 - "${REPO_ROOT}" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1])
readme = (root / "README.md").read_text(encoding="utf-8")
run_sh = (root / "scripts/run.sh").read_text(encoding="utf-8")
stop_sh = (root / "scripts/stop.sh").read_text(encoding="utf-8")

required_readme = [
    "docker compose -f infra/docker-compose.yml --env-file .env up --build",
    "docker compose -f infra/docker-compose.yml --env-file .env down",
    "scripts/runtime-lifecycle-check.sh --check",
]
missing = [item for item in required_readme if item not in readme]
if missing:
    raise SystemExit("README is missing lifecycle command text: " + ", ".join(missing))

if "igy6_run_compose up --build" not in run_sh:
    raise SystemExit("scripts/run.sh must use up --build")
if "igy6_run_compose down" not in stop_sh:
    raise SystemExit("scripts/stop.sh must use down")

for label, text in (("scripts/run.sh", run_sh), ("scripts/stop.sh", stop_sh), ("README.md", readme)):
    forbidden = ["down -v", "rm -rf", "docker volume rm", "docker system prune"]
    found = [item for item in forbidden if item in text and item != "down -v"]
    if found:
        raise SystemExit(f"{label} contains destructive lifecycle text: {', '.join(found)}")
PY
}

assert_manifest_lifecycle_posture() {
  python3 - "${MANIFEST}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = json.load(handle)

final_audit = manifest.get("final_rust_only_runtime_audit", {})
lifecycle = manifest.get("runtime_lifecycle_validation", {})

checks = {
    "target_architecture": manifest.get("target_architecture") == "rust-only-application-runtime",
    "fastapi_fallback_required": manifest.get("fastapi_fallback_required") is False,
    "python_celery_worker_active": final_audit.get("python_celery_worker_active") is False,
    "python_celery_beat_active": final_audit.get("python_celery_beat_active") is False,
    "rust_only_application_runtime_claimed": final_audit.get("rust_only_application_runtime_claimed") is True,
    "lifecycle_command": lifecycle.get("command") == "scripts/runtime-lifecycle-check.sh --check",
    "lifecycle_non_destructive": lifecycle.get("default_non_destructive") is True,
    "runtime_ownership_changed": lifecycle.get("runtime_ownership_changed") is False,
}

failed = [name for name, ok in checks.items() if not ok]
if failed:
    raise SystemExit(f"manifest lifecycle posture checks failed: {', '.join(failed)}")
PY
}

assert_rollback_posture() {
  python3 - "${MANIFEST}" "${REPO_ROOT}/README.md" <<'PY'
import json
import sys
from pathlib import Path

with open(sys.argv[1], encoding="utf-8") as handle:
    manifest = json.load(handle)
readme = Path(sys.argv[2]).read_text(encoding="utf-8")

rollback = manifest.get("final_rust_only_runtime_audit", {}).get("rollback_posture", "")
required = [
    "archive/legacy-python/services-worker",
    "restore the prior Python/Celery worker",
    "validate Docker Compose",
]
missing_manifest = [item for item in required if item not in rollback]
missing_readme = [item for item in ("archive/legacy-python/services-worker", "docker compose -f infra/docker-compose.yml --env-file .env config") if item not in readme]
if missing_manifest:
    raise SystemExit("manifest rollback posture is missing: " + ", ".join(missing_manifest))
if missing_readme:
    raise SystemExit("README rollback posture is missing: " + ", ".join(missing_readme))
PY
}

print_lifecycle_plan() {
  cat <<'EOF'
Lifecycle command plan:
  start:
    docker compose -f infra/docker-compose.yml --env-file .env up --build
  shutdown:
    docker compose -f infra/docker-compose.yml --env-file .env down
  restart:
    docker compose -f infra/docker-compose.yml --env-file .env down
    docker compose -f infra/docker-compose.yml --env-file .env up --build -d
EOF
}

run_check() {
  cd "${REPO_ROOT}"

  check "tool availability"
  require_command docker
  require_command python3

  run_required "Docker Compose lifecycle posture is valid" assert_compose_lifecycle_posture
  run_required "lifecycle command shapes are documented and safe" assert_lifecycle_command_shapes
  run_required "manifest records lifecycle validation posture" assert_manifest_lifecycle_posture
  run_required "rollback posture remains documented" assert_rollback_posture
  run_required "post-cutover runtime audit passes" python3 "${REPO_ROOT}/scripts/post-cutover-runtime-audit.py"
  run_required "post-cutover smoke suite passes" "${REPO_ROOT}/scripts/post-cutover-smoke.sh" --check
  print_lifecycle_plan

  if [[ "${FAILURES}" -gt 0 ]]; then
    exit 1
  fi
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
