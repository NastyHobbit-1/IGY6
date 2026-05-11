#!/usr/bin/env bash

set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=lib/igy6-ops.sh
source "${SCRIPT_DIR}/lib/igy6-ops.sh"

usage() {
  cat <<'EOF'
Usage: scripts/run-last-healthy-config.sh [--help]

Start IGY6 detached using the last known healthy local Compose metadata.

The snapshot is written by:
  scripts/run.sh --detached

Safety:
  - Refuses if no snapshot exists.
  - Shows snapshot metadata before starting.
  - Warns if the current git commit differs from the snapshot.
  - Warns if the working tree has local changes.
  - Does not checkout, reset, stash, or overwrite files.
  - Does not modify .env.
  - Does not delete volumes, images, or runtime data.
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
igy6_snapshot_summary
igy6_validate_snapshot_paths

snapshot_commit="$(igy6_snapshot_commit)"
current_commit="$(igy6_git_commit)"
if [[ "${snapshot_commit}" != "${current_commit}" ]]; then
  igy6_warn "Current git commit differs from snapshot commit."
  igy6_warn "  snapshot: ${snapshot_commit}"
  igy6_warn "  current:  ${current_commit}"
fi

if igy6_git_dirty; then
  igy6_warn "Working tree has local changes. The script will not reset, stash, or discard them."
fi

igy6_check_ports
igy6_info "Starting detached from validated last-healthy Compose/env metadata."
igy6_run_compose up --build -d
