#!/usr/bin/env bash

set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/igy6-ops.sh
source "${SCRIPT_DIR}/lib/igy6-ops.sh"

usage() {
  cat <<'EOF'
Usage: scripts/run.sh [--detached] [--help]

Start the local IGY6 Docker Compose stack.

Default:
  Runs in the foreground with:
  docker compose -f infra/docker-compose.yml --env-file .env up --build

Options:
  --detached   Start with -d, run health checks, and write last-healthy metadata.
  --help       Show this help.

Safety:
  - Does not modify .env.
  - Does not delete volumes or images.
  - Warns about common host ports but never kills processes.
  - Stores only safe operational metadata after detached health checks pass.
EOF
}

detached=false
for arg in "$@"; do
  case "${arg}" in
    --detached)
      detached=true
      ;;
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
igy6_check_ports

if [[ "${detached}" == "true" ]]; then
  igy6_run_compose up --build -d
  if igy6_health_checks; then
    igy6_write_snapshot
  else
    igy6_die "Detached startup health checks failed; last-healthy snapshot was not updated."
  fi
else
  igy6_info "Foreground mode selected. Last-healthy snapshot is written only by --detached after health checks pass."
  igy6_run_compose up --build
fi
