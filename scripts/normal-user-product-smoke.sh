#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
PAGE_FILE="${REPO_ROOT}/apps/web/src/app/page.tsx"

usage() {
  cat <<'EOF'
Usage: scripts/normal-user-product-smoke.sh --check
       scripts/normal-user-product-smoke.sh --owner-commands
       scripts/normal-user-product-smoke.sh --help

Modes:
  --check
    Non-Docker Codex-safe check that verifies the normal-user product smoke
    surfaces are present in the web UI source. It does not start services,
    create records, read .env, or touch runtime data.

  --owner-commands
    Print the owner-run WSL commands for live synthetic product-path smoke.

Safety:
  - Synthetic data only.
  - No Docker command is run by --check.
  - No runtime/private data is dumped.
  - Passing --check does not mean the live product path passed.
EOF
}

require_marker() {
  local label="$1"
  local marker="$2"
  if grep -Fq "${marker}" "${PAGE_FILE}"; then
    printf 'PASS %s marker present: %s\n' "${label}" "${marker}"
  else
    printf 'FAIL %s marker missing: %s\n' "${label}" "${marker}" >&2
    return 1
  fi
}

check_markers() {
  local failures=0
  [[ -f "${PAGE_FILE}" ]] || {
    printf 'FAIL missing UI source file: apps/web/src/app/page.tsx\n' >&2
    return 1
  }

  require_marker "Add Data guided source/upload" "data-guided-manual-upload" || failures=$((failures + 1))
  require_marker "Work processing status" "data-work-status-item" || failures=$((failures + 1))
  require_marker "Results evidence answer" "data-chat-preview-results" || failures=$((failures + 1))
  require_marker "Persisted answer save" "data-chat-save-answer" || failures=$((failures + 1))
  require_marker "Report workflow" "data-basic-report-workflow" || failures=$((failures + 1))
  require_marker "Feedback and outcome workflow" "data-evidence-feedback-workflow" || failures=$((failures + 1))
  require_marker "Outcome form" "data-outcome-form" || failures=$((failures + 1))
  require_marker "Source/evidence detail review" "data-source-evidence-history" || failures=$((failures + 1))

  if [[ "${failures}" -gt 0 ]]; then
    printf 'FAIL normal-user product smoke source marker check found %s missing marker(s)\n' "${failures}" >&2
    return 1
  fi

  printf 'PASS normal-user product smoke source marker check passed\n'
  printf 'INFO live product-path verification still requires owner WSL smoke with synthetic data.\n'
}

print_owner_commands() {
  cat <<'EOF'
Owner-run WSL live product smoke commands:

  scripts/operator-smoke-check.sh --check
  scripts/operator-smoke-check.sh --run --record
  scripts/operator-smoke-check.sh --latest-result

Then follow docs/runtime/NORMAL_USER_PRODUCT_SMOKE.md with synthetic text only:

  source title: DIFF-232 Synthetic Product Smoke
  source type: manual_upload
  sample question: What did the DIFF-232 smoke note say needs review?

Do not use private runtime data for this smoke.
EOF
}

case "${1:-}" in
  --check)
    check_markers
    ;;
  --owner-commands)
    print_owner_commands
    ;;
  --help|-h|"")
    usage
    ;;
  *)
    printf 'FAIL unknown argument: %s\n' "$1" >&2
    usage >&2
    exit 1
    ;;
esac
