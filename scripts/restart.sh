#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/igy6-ops.sh
source "${SCRIPT_DIR}/lib/igy6-ops.sh"

usage() {
  cat <<'EOF'
Usage: scripts/restart.sh [--help]

Restart the local IGY6 Docker Compose stack.

Runs:
  docker compose -f infra/docker-compose.yml --env-file .env down
  docker compose -f infra/docker-compose.yml --env-file .env up --build

Safety:
  - Does not use down -v.
  - Does not remove volumes.
  - Does not remove images.
  - Does not delete runtime data.
  - Does not modify or create .env.
  - Prints Docker Compose errors directly.
EOF
}

for arg in "$@"; do
  case "${arg}" in
    --help|-h)
      usage
      exit 0
      ;;
    *)
      igy6_die "Unknown argument: ${arg}. Use --help."
      ;;
  esac
done

igy6_require_repo_files
igy6_require_docker_compose
igy6_info "Restarting IGY6 local stack. Persistent data under IGY6_DATA_ROOT is preserved."
igy6_info "Stopping existing containers without removing volumes..."
igy6_run_compose down
igy6_info "Starting the stack again..."
igy6_run_compose up --build
