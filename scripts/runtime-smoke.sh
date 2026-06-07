#!/usr/bin/env bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COMPOSE_FILE="${REPO_ROOT}/infra/docker-compose.yml"
ENV_FILE="${IGY6_ENV_FILE:-${REPO_ROOT}/.env}"

if [[ ! -f "${ENV_FILE}" ]]; then
  ENV_FILE="${REPO_ROOT}/.env.example"
fi

MODE="check"
DETACHED=0
FAILURES=0

EXPECTED_SERVICES=(
  postgres
  qdrant
  neo4j
  mlflow
  phoenix
  api
  worker
  web
)

usage() {
  cat <<EOF
IGY6 runtime smoke check

Usage:
  scripts/runtime-smoke.sh [--check] [--start] [--stop] [--detached] [--help]

Default:
  --check   Validate compose config and check an already-running local stack.

Options:
  --check      Check compose config, expected running services, API health, and web.
  --start      Explicitly start the stack with docker compose up --build.
  --stop       Explicitly stop the stack with docker compose down. Never uses down -v.
  --detached   With --start, run docker compose up --build --detach.
  --help       Show this help.

Environment:
  IGY6_ENV_FILE  Optional env file path. Defaults to .env when present, otherwise .env.example.

Diagnostics:
  docker compose -f infra/docker-compose.yml --env-file .env ps
  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web
EOF
}

pass() {
  printf 'PASS %s\n' "$1"
}

fail() {
  printf 'FAIL %s\n' "$1"
  FAILURES=$((FAILURES + 1))
}

note_diagnostics() {
  cat <<EOF

Next diagnostic commands:
  docker compose -f infra/docker-compose.yml --env-file .env ps
  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api
  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web
EOF
}

compose() {
  docker compose -f "${COMPOSE_FILE}" --env-file "${ENV_FILE}" "$@"
}

require_command() {
  if command -v "$1" >/dev/null 2>&1; then
    pass "$1 is available"
  else
    fail "$1 is not available"
  fi
}

check_compose_config() {
  if compose config >/dev/null; then
    pass "docker compose config is valid using ${ENV_FILE}"
  else
    fail "docker compose config failed using ${ENV_FILE}"
  fi
}

check_running_services() {
  local running
  running="$(compose ps --services --filter status=running 2>/dev/null || true)"

  if [[ -z "${running}" ]]; then
    fail "no running compose services found; start the stack before --check"
    return
  fi

  local service
  for service in "${EXPECTED_SERVICES[@]}"; do
    if grep -Fxq "${service}" <<<"${running}"; then
      pass "service ${service} is running"
    else
      fail "service ${service} is not running"
    fi
  done
}

http_status() {
  local url="$1"
  curl --silent --show-error --output /dev/null --write-out '%{http_code}' --max-time 8 "${url}" 2>/dev/null || true
}

check_http_ok() {
  local label="$1"
  local url="$2"
  local status
  status="$(http_status "${url}")"
  case "${status}" in
    2*|3*)
      pass "${label} responded with HTTP ${status}"
      ;;
    000|"")
      fail "${label} did not respond at ${url}"
      ;;
    *)
      fail "${label} responded with HTTP ${status} at ${url}"
      ;;
  esac
}

check_http_available() {
  local label="$1"
  local url="$2"
  local status
  status="$(http_status "${url}")"
  case "${status}" in
    2*|3*)
      pass "${label} responded healthy with HTTP ${status}"
      ;;
    000|"")
      fail "${label} did not respond at ${url}"
      ;;
    *)
      fail "${label} responded but is not ready: HTTP ${status}"
      ;;
  esac
}

run_check() {
  require_command docker
  require_command curl
  check_compose_config
  check_running_services
  local api_port web_port
  api_port="$(grep -E '^APP_PORT=' "${ENV_FILE}" | tail -n1 | cut -d= -f2- | tr -d '\r' || true)"
  web_port="$(grep -E '^WEB_PORT=' "${ENV_FILE}" | tail -n1 | cut -d= -f2- | tr -d '\r' || true)"
  api_port="${api_port:-8000}"
  web_port="${web_port:-3000}"
  check_http_ok "API live" "http://127.0.0.1:${api_port}/health/live"
  check_http_available "API ready" "http://127.0.0.1:${api_port}/health/ready"
  check_http_ok "Web UI" "http://127.0.0.1:${web_port}"

  if [[ "${FAILURES}" -gt 0 ]]; then
    note_diagnostics
    exit 1
  fi
}

run_start() {
  require_command docker
  check_compose_config
  if [[ "${FAILURES}" -gt 0 ]]; then
    note_diagnostics
    exit 1
  fi
  if [[ "${DETACHED}" -eq 1 ]]; then
    compose up --build --detach
  else
    compose up --build
  fi
}

run_stop() {
  require_command docker
  check_compose_config
  if [[ "${FAILURES}" -gt 0 ]]; then
    note_diagnostics
    exit 1
  fi
  compose down
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --check)
      MODE="check"
      ;;
    --start)
      MODE="start"
      ;;
    --stop)
      MODE="stop"
      ;;
    --detached)
      DETACHED=1
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
  start)
    run_start
    ;;
  stop)
    run_stop
    ;;
esac
