#!/usr/bin/env bash

set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/igy6-ops.sh
source "${SCRIPT_DIR}/lib/igy6-ops.sh"

usage() {
  cat <<'EOF'
Usage: scripts/stop.sh [--help]

Stop the local IGY6 Docker Compose stack.

Runs:
  docker compose -f infra/docker-compose.yml --env-file .env down

Safety:
  - Does not remove volumes.
  - Does not remove images.
  - Does not prune Docker.
  - Preserves persistent data under IGY6_DATA_ROOT.
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
igy6_info "Persistent data under IGY6_DATA_ROOT is preserved. No volumes, images, or data folders will be deleted."
igy6_run_compose down
