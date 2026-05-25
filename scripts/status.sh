#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/igy6-ops.sh
source "${SCRIPT_DIR}/lib/igy6-ops.sh"

usage() {
  cat <<'EOF'
Usage: scripts/status.sh [--help]

Show the current IGY6 Docker Compose service status.

Runs:
  docker compose -f infra/docker-compose.yml --env-file .env ps

Safety:
  - Does not start or stop services.
  - Does not remove volumes.
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
igy6_info "Showing IGY6 local stack status..."
igy6_run_compose ps
