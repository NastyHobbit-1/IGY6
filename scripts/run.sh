#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/igy6-ops.sh
source "${SCRIPT_DIR}/lib/igy6-ops.sh"

usage() {
  cat <<'EOF'
Usage: scripts/run.sh [--help]

Start the local IGY6 Docker Compose stack.

Runs:
  docker compose -f infra/docker-compose.yml --env-file .env up --build

Options:
  --help  Show this help.

Safety:
  - On first run (missing .env), will automatically create .env from .env.example + create IGY6_Data under $HOME with grok-branch defaults (absolute path, SINGLE_USER_MODE, helpful comments).
  - Does not delete volumes or images.
  - Does not overwrite an existing .env.
  - Prints Docker Compose errors directly.
  - After bootstrap, all deep collection, Media Library, password/TOTP linking, etc. is done from the web UI (no further cmdline needed).
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

igy6_info "Starting IGY6 local stack (detached), then streaming logs. Press Ctrl+C to stop log streaming; containers keep running."
igy6_run_compose up --build -d
igy6_init_db_schema
igy6_run_compose logs -f
