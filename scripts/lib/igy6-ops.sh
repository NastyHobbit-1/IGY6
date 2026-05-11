#!/usr/bin/env bash

set -Eeuo pipefail

IGY6_OPS_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IGY6_REPO_ROOT="$(cd "${IGY6_OPS_LIB_DIR}/../.." && pwd)"
IGY6_COMPOSE_FILE="${IGY6_REPO_ROOT}/infra/docker-compose.yml"
IGY6_ENV_FILE="${IGY6_REPO_ROOT}/.env"

IGY6_PORTS=(3000 8000 5432 6379 6333 7474 7687 5000 6006)
IGY6_HEALTH_ENDPOINTS=(
  "api_ready|http://127.0.0.1:8000/health/ready|required"
  "web_ui|http://127.0.0.1:3000|required"
  "qdrant|http://127.0.0.1:6333|optional"
  "phoenix|http://127.0.0.1:6006|optional"
  "mlflow|http://127.0.0.1:5000|optional"
)

igy6_die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

igy6_info() {
  printf '%s\n' "$*"
}

igy6_warn() {
  printf 'WARN: %s\n' "$*" >&2
}

igy6_require_repo_files() {
  [[ -f "${IGY6_COMPOSE_FILE}" ]] || igy6_die "Missing Compose file: ${IGY6_COMPOSE_FILE}"
  [[ -f "${IGY6_ENV_FILE}" ]] || igy6_die "Missing .env file: ${IGY6_ENV_FILE}. Create it with: cp .env.example .env"
}

igy6_require_docker_compose() {
  command -v docker >/dev/null 2>&1 || igy6_die "Docker CLI is not available on PATH."
  docker compose version >/dev/null 2>&1 || igy6_die "Docker Compose plugin is unavailable. Install Docker Compose v2."
}

igy6_compose_cmd() {
  printf '%q ' docker compose -f "${IGY6_COMPOSE_FILE}" --env-file "${IGY6_ENV_FILE}" "$@"
  printf '\n'
}

igy6_run_compose() {
  igy6_info "Running Docker Compose command:"
  igy6_compose_cmd "$@"
  docker compose -f "${IGY6_COMPOSE_FILE}" --env-file "${IGY6_ENV_FILE}" "$@"
}

igy6_env_value() {
  local key="$1"
  local line value
  line="$(grep -E "^[[:space:]]*(export[[:space:]]+)?${key}=" "${IGY6_ENV_FILE}" | tail -n 1 || true)"
  [[ -n "${line}" ]] || return 1
  value="${line#*=}"
  value="${value%%#*}"
  value="${value%"${value##*[![:space:]]}"}"
  value="${value#"${value%%[![:space:]]*}"}"
  if [[ "${value}" == \"*\" && "${value}" == *\" ]]; then
    value="${value:1:${#value}-2}"
  elif [[ "${value}" == \'*\' && "${value}" == *\' ]]; then
    value="${value:1:${#value}-2}"
  fi
  printf '%s\n' "${value}"
}

igy6_data_root() {
  local value resolved
  value="$(igy6_env_value "IGY6_DATA_ROOT" || true)"
  [[ -n "${value}" ]] || igy6_die "IGY6_DATA_ROOT is not set in .env; refusing to write operator metadata."
  if [[ "${value}" = /* ]]; then
    resolved="${value}"
  elif [[ "${value}" =~ ^[A-Za-z]:/ ]]; then
    resolved="${value}"
  else
    resolved="$(cd "${IGY6_REPO_ROOT}" && python3 -c 'import os,sys; print(os.path.abspath(sys.argv[1]))' "${value}")"
  fi
  [[ -n "${resolved}" ]] || igy6_die "Could not resolve IGY6_DATA_ROOT."
  printf '%s\n' "${resolved}"
}

igy6_snapshot_path() {
  local root
  root="$(igy6_data_root)"
  printf '%s/ops/last-healthy.json\n' "${root}"
}

igy6_sha256() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${path}" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${path}" | awk '{print $1}'
  else
    igy6_die "No sha256sum or shasum command is available."
  fi
}

igy6_json_tool() {
  if command -v python3 >/dev/null 2>&1; then
    printf 'python3\n'
    return
  fi
  if command -v python >/dev/null 2>&1; then
    printf 'python\n'
    return
  fi
  if command -v jq >/dev/null 2>&1; then
    printf 'jq\n'
    return
  fi
  igy6_die "Need python3, python, or jq for JSON snapshot handling."
}

igy6_git_commit() {
  git -C "${IGY6_REPO_ROOT}" rev-parse HEAD 2>/dev/null || printf 'unknown'
}

igy6_git_branch() {
  git -C "${IGY6_REPO_ROOT}" branch --show-current 2>/dev/null || printf 'unknown'
}

igy6_git_dirty() {
  [[ -n "$(git -C "${IGY6_REPO_ROOT}" status --short 2>/dev/null || true)" ]]
}

igy6_check_ports() {
  local port
  igy6_info "Checking common IGY6 host ports for listeners..."
  for port in "${IGY6_PORTS[@]}"; do
    if command -v ss >/dev/null 2>&1; then
      if ss -ltn "( sport = :${port} )" 2>/dev/null | grep -q ":${port}"; then
        igy6_warn "Port ${port} already appears to be listening. The script will not kill any process."
      fi
    elif command -v lsof >/dev/null 2>&1; then
      if lsof -iTCP:"${port}" -sTCP:LISTEN >/dev/null 2>&1; then
        igy6_warn "Port ${port} already appears to be listening. The script will not kill any process."
      fi
    elif command -v netstat >/dev/null 2>&1; then
      if netstat -ltn 2>/dev/null | grep -q ":${port} "; then
        igy6_warn "Port ${port} already appears to be listening. The script will not kill any process."
      fi
    else
      igy6_warn "No ss, lsof, or netstat command found; skipping port listener checks."
      return
    fi
  done
}

igy6_http_check() {
  local name="$1"
  local url="$2"
  local attempts="${3:-30}"
  local delay="${4:-2}"
  local attempt status
  for ((attempt = 1; attempt <= attempts; attempt++)); do
    status="$(curl -fsS --max-time 5 -o /dev/null -w '%{http_code}' "${url}" 2>/dev/null || true)"
    if [[ "${status}" =~ ^[23] ]]; then
      printf '%s|%s|pass|HTTP %s\n' "${name}" "${url}" "${status}"
      return 0
    fi
    sleep "${delay}"
  done
  printf '%s|%s|fail|last HTTP %s\n' "${name}" "${url}" "${status:-none}"
  return 1
}

igy6_health_checks() {
  local failures=0
  local endpoint name url required result
  IGY6_HEALTH_RESULTS=()
  for endpoint in "${IGY6_HEALTH_ENDPOINTS[@]}"; do
    IFS='|' read -r name url required <<< "${endpoint}"
    igy6_info "Health check: ${name} ${url}"
    if result="$(igy6_http_check "${name}" "${url}")"; then
      igy6_info "PASS ${result}"
      IGY6_HEALTH_RESULTS+=("${result}")
    else
      igy6_warn "FAIL ${result}"
      IGY6_HEALTH_RESULTS+=("${result}")
      [[ "${required}" == "required" ]] && failures=$((failures + 1))
    fi
  done
  [[ "${failures}" -eq 0 ]]
}

igy6_service_summary_json() {
  docker compose -f "${IGY6_COMPOSE_FILE}" --env-file "${IGY6_ENV_FILE}" ps --format json 2>/dev/null || printf '[]\n'
}

igy6_write_snapshot() {
  local snapshot_path data_root json_tool timestamp compose_hash env_hash compose_version service_json health_lines
  snapshot_path="$(igy6_snapshot_path)"
  data_root="$(igy6_data_root)"
  json_tool="$(igy6_json_tool)"
  [[ "${json_tool}" != "jq" ]] || igy6_die "Python is required to write the snapshot without risking shell JSON escaping issues."

  mkdir -p "$(dirname "${snapshot_path}")"
  timestamp="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  compose_hash="$(igy6_sha256 "${IGY6_COMPOSE_FILE}")"
  env_hash="$(igy6_sha256 "${IGY6_ENV_FILE}")"
  compose_version="$(docker compose version 2>/dev/null || printf 'unknown')"
  service_json="$(igy6_service_summary_json)"
  health_lines="$(printf '%s\n' "${IGY6_HEALTH_RESULTS[@]:-}")"

  SNAPSHOT_PATH="${snapshot_path}" \
  SNAPSHOT_TIMESTAMP="${timestamp}" \
  SNAPSHOT_GIT_COMMIT="$(igy6_git_commit)" \
  SNAPSHOT_GIT_BRANCH="$(igy6_git_branch)" \
  SNAPSHOT_REPO_ROOT="${IGY6_REPO_ROOT}" \
  SNAPSHOT_COMPOSE_HASH="${compose_hash}" \
  SNAPSHOT_ENV_HASH="${env_hash}" \
  SNAPSHOT_DATA_ROOT="${data_root}" \
  SNAPSHOT_COMPOSE_VERSION="${compose_version}" \
  SNAPSHOT_SERVICE_JSON="${service_json}" \
  SNAPSHOT_HEALTH_LINES="${health_lines}" \
  "${json_tool}" - <<'PY'
import json
import os
from pathlib import Path

snapshot_path = Path(os.environ["SNAPSHOT_PATH"])
service_text = os.environ["SNAPSHOT_SERVICE_JSON"]
try:
    parsed = json.loads(service_text) if service_text.strip() else []
    raw_services = parsed if isinstance(parsed, list) else [parsed]
except json.JSONDecodeError:
    raw_services = []
    for line in service_text.splitlines():
        line = line.strip()
        if not line:
            continue
        try:
            raw_services.append(json.loads(line))
        except json.JSONDecodeError:
            raw_services.append({"raw": line})

services = []
for item in raw_services:
    if not isinstance(item, dict):
        continue
    services.append({
        "service": item.get("Service") or item.get("Name") or item.get("service"),
        "name": item.get("Name") or item.get("Names"),
        "state": item.get("State"),
        "status": item.get("Status"),
        "health": item.get("Health"),
        "ports": item.get("Ports"),
    })

health = []
for line in os.environ["SNAPSHOT_HEALTH_LINES"].splitlines():
    parts = line.split("|", 3)
    if len(parts) == 4:
        health.append({
            "name": parts[0],
            "url": parts[1],
            "status": parts[2],
            "detail": parts[3],
        })

payload = {
    "schema_version": 1,
    "timestamp_utc": os.environ["SNAPSHOT_TIMESTAMP"],
    "git_commit": os.environ["SNAPSHOT_GIT_COMMIT"],
    "git_branch": os.environ["SNAPSHOT_GIT_BRANCH"],
    "repo_root": os.environ["SNAPSHOT_REPO_ROOT"],
    "compose_file_relative_path": "infra/docker-compose.yml",
    "env_file_relative_path": ".env",
    "env_file_sha256": os.environ["SNAPSHOT_ENV_HASH"],
    "compose_file_sha256": os.environ["SNAPSHOT_COMPOSE_HASH"],
    "data_root_path": os.environ["SNAPSHOT_DATA_ROOT"],
    "docker_compose_version": os.environ["SNAPSHOT_COMPOSE_VERSION"],
    "services": services,
    "health_checks": health,
}
snapshot_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
  igy6_info "Wrote last healthy snapshot: ${snapshot_path}"
}

igy6_snapshot_summary() {
  local snapshot_path json_tool
  snapshot_path="$(igy6_snapshot_path)"
  [[ -f "${snapshot_path}" ]] || igy6_die "No last healthy snapshot exists at ${snapshot_path}"
  json_tool="$(igy6_json_tool)"
  [[ "${json_tool}" != "jq" ]] || igy6_die "Python is required to read the snapshot safely."
  SNAPSHOT_PATH="${snapshot_path}" "${json_tool}" - <<'PY'
import json
import os
from pathlib import Path

payload = json.loads(Path(os.environ["SNAPSHOT_PATH"]).read_text(encoding="utf-8"))
print(f"Snapshot: {os.environ['SNAPSHOT_PATH']}")
print(f"timestamp_utc: {payload.get('timestamp_utc')}")
print(f"git_commit: {payload.get('git_commit')}")
print(f"git_branch: {payload.get('git_branch')}")
print(f"compose_file: {payload.get('compose_file_relative_path')}")
print(f"env_file: {payload.get('env_file_relative_path')}")
print(f"data_root_path: {payload.get('data_root_path')}")
print(f"docker_compose_version: {payload.get('docker_compose_version')}")
print("health_checks:")
for check in payload.get("health_checks", []):
    print(f"  - {check.get('name')}: {check.get('status')} ({check.get('detail')}) {check.get('url')}")
PY
}

igy6_snapshot_commit() {
  local snapshot_path json_tool
  snapshot_path="$(igy6_snapshot_path)"
  json_tool="$(igy6_json_tool)"
  [[ "${json_tool}" != "jq" ]] || igy6_die "Python is required to read the snapshot safely."
  SNAPSHOT_PATH="${snapshot_path}" "${json_tool}" - <<'PY'
import json
import os
from pathlib import Path
print(json.loads(Path(os.environ["SNAPSHOT_PATH"]).read_text(encoding="utf-8")).get("git_commit", "unknown"))
PY
}

igy6_validate_snapshot_paths() {
  local snapshot_path json_tool
  snapshot_path="$(igy6_snapshot_path)"
  [[ -f "${snapshot_path}" ]] || igy6_die "No last healthy snapshot exists at ${snapshot_path}"
  json_tool="$(igy6_json_tool)"
  [[ "${json_tool}" != "jq" ]] || igy6_die "Python is required to validate the snapshot safely."
  SNAPSHOT_PATH="${snapshot_path}" CURRENT_REPO_ROOT="${IGY6_REPO_ROOT}" "${json_tool}" - <<'PY'
import json
import os
from pathlib import Path

payload = json.loads(Path(os.environ["SNAPSHOT_PATH"]).read_text(encoding="utf-8"))
repo_root = Path(os.environ["CURRENT_REPO_ROOT"]).resolve()
for key in ("compose_file_relative_path", "env_file_relative_path"):
    value = payload.get(key)
    if not isinstance(value, str) or value.startswith("/") or ".." in Path(value).parts:
        raise SystemExit(f"Unsafe snapshot path metadata for {key}: {value!r}")
    target = (repo_root / value).resolve()
    try:
        target.relative_to(repo_root)
    except ValueError:
        raise SystemExit(f"Snapshot path escapes repository: {value}")
    if not target.is_file():
        raise SystemExit(f"Snapshot path no longer exists: {target}")
print("Snapshot paths validated inside current repository.")
PY
}
