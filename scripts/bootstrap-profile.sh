#!/usr/bin/env bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"
PROFILES_DIR="${REPO_ROOT}/configs/profiles"

usage() {
  cat <<'EOF'
Usage:
  scripts/bootstrap-profile.sh [--wizard] [quick-start|standard|advanced|expert]

Applies a configuration profile to .env idempotently (adds or updates known keys).
Profiles live under configs/profiles/*.env. Existing unrelated keys are preserved.

Options:
  --wizard   Interactive selection (safe re-run; records last choice).
  --help     Show help.

Profiles:
  quick-start   Single-user, approvals on, external models blocked (recommended)
  standard      Single-user, approvals on, external models blocked
  advanced      Single-user, approvals on, with advanced knobs (commented)
  expert        Multi-user capable; external models still blocked by default
EOF
}

apply_profile() {
  local profile_name="$1"
  local profile_path="${PROFILES_DIR}/${profile_name}.env"
  [[ -f "${profile_path}" ]] || { echo "ERROR: Unknown profile: ${profile_name}" >&2; exit 1; }
  [[ -f "${ENV_FILE}" ]] || { echo "ERROR: Missing .env at ${ENV_FILE}. Create it first (e.g. igy6 start)." >&2; exit 1; }
  echo "Applying profile '${profile_name}' to ${ENV_FILE} ..."
  # For each KEY=VALUE line (non-comment) set or update the key in .env
  while IFS= read -r line; do
    [[ -n "${line}" ]] || continue
    [[ "${line}" =~ ^[[:space:]]*# ]] && continue
    key="${line%%=*}"
    value="${line#*=}"
    # Escape for sed
    esc_key="$(printf '%s' "${key}" | sed -e 's/[^^]/[&]/g; s/\\^/\\\\^/g')"
    esc_val="$(printf '%s' "${value}" | sed -e 's/[&/]/\\&/g')"
    if grep -qE "^[[:space:]]*(${esc_key})=" "${ENV_FILE}"; then
      sed -i -E "s|^[[:space:]]*(${esc_key})=.*|\1=${esc_val}|" "${ENV_FILE}"
    else
      printf '%s=%s\n' "${key}" "${value}" >> "${ENV_FILE}"
    fi
  done < "${profile_path}"
  echo "Profile '${profile_name}' applied."
  # Record last applied
  local data_root
  data_root="$(grep -E '^[[:space:]]*(export[[:space:]]+)?IGY6_DATA_ROOT=' "${ENV_FILE}" | tail -n1 | cut -d'=' -f2- | tr -d '\"')"
  [[ -n "${data_root}" ]] || data_root="${REPO_ROOT}/storage"
  mkdir -p "${data_root}/ops"
  printf '{ "profile": "%s" }\n' "${profile_name}" > "${data_root}/ops/installer-profile.json"
  echo "Recorded last applied profile to ${data_root}/ops/installer-profile.json"
}

select_profile_wizard() {
  echo "Select a configuration profile:"
  select choice in quick-start standard advanced expert cancel; do
    case "${choice:-}" in
      quick-start|standard|advanced|expert) apply_profile "${choice}"; break ;;
      cancel) echo "Canceled."; exit 0 ;;
      *) echo "Invalid choice." ;;
    esac
  done
}

main() {
  if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
    usage; exit 0
  fi
  if [[ "${1:-}" == "--wizard" ]]; then
    select_profile_wizard; exit 0
  fi
  local profile="${1:-quick-start}"
  apply_profile "${profile}"
}

main "$@"

