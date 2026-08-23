#!/usr/bin/env bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ENV_FILE="${REPO_ROOT}/.env"
PROFILES_DIR="${REPO_ROOT}/configs/profiles"

usage() {
  cat <<'EOF'
Usage:
  scripts/bootstrap-profile.sh [--wizard|--check] [quick-start|standard|advanced|expert]

Applies a configuration profile to .env idempotently (adds or updates known keys).
Profiles live under configs/profiles/*.env. Existing unrelated keys are preserved.

Options:
  --wizard   Interactive selection (safe re-run; records last choice).
  --check    Print environment readiness summary (no changes).
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
  # Backup current .env (recoverable/rollback)
  if [[ -f "${ENV_FILE}" ]]; then
    local data_root
    data_root="$(grep -E '^[[:space:]]*(export[[:space:]]+)?IGY6_DATA_ROOT=' "${ENV_FILE}" | tail -n1 | cut -d'=' -f2- | tr -d '\"')"
    local backup_base
    if [[ -n "${data_root}" ]]; then
      backup_base="${data_root}/ops/env-backups"
    else
      backup_base="${REPO_ROOT}/.igy6-backups/env"
    fi
    mkdir -p "${backup_base}"
    local ts
    ts="$(date -u +"%Y%m%dT%H%M%SZ")"
    cp -f "${ENV_FILE}" "${backup_base}/env-${ts}.bak"
    echo "Backup written: ${backup_base}/env-${ts}.bak"
  fi
  # Atomic apply: work on a temp copy, then move into place
  local tmp_env
  tmp_env="${ENV_FILE}.tmp.$$"
  cp -f "${ENV_FILE}" "${tmp_env}"
  trap 'rm -f "'"${tmp_env}"'" 2>/dev/null || true' EXIT
  # For each KEY=VALUE line (non-comment) set or update the key in temp
  while IFS= read -r line; do
    [[ -n "${line}" ]] || continue
    [[ "${line}" =~ ^[[:space:]]*# ]] && continue
    key="${line%%=*}"
    value="${line#*=}"
    # Escape for sed
    esc_key="$(printf '%s' "${key}" | sed -e 's/[^^]/[&]/g; s/\\^/\\\\^/g')"
    esc_val="$(printf '%s' "${value}" | sed -e 's/[&/]/\\&/g')"
    if grep -qE "^[[:space:]]*(${esc_key})=" "${tmp_env}"; then
      sed -i -E "s|^[[:space:]]*(${esc_key})=.*|\1=${esc_val}|" "${tmp_env}"
    else
      printf '%s=%s\n' "${key}" "${value}" >> "${tmp_env}"
    fi
  done < "${profile_path}"
  # Move into place (atomic on same filesystem)
  mv -f "${tmp_env}" "${ENV_FILE}"
  trap - EXIT
  echo "Profile '${profile_name}' applied."
  # Record last applied
  local data_root
  data_root="$(grep -E '^[[:space:]]*(export[[:space:]]+)?IGY6_DATA_ROOT=' "${ENV_FILE}" | tail -n1 | cut -d'=' -f2- | tr -d '\"')"
  [[ -n "${data_root}" ]] || data_root="${REPO_ROOT}/storage"
  mkdir -p "${data_root}/ops"
  printf '{ "profile": "%s" }\n' "${profile_name}" > "${data_root}/ops/installer-profile.json"
  echo "Recorded last applied profile to ${data_root}/ops/installer-profile.json"
  echo "Note: Restart the IGY6 stack to apply environment changes."
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
  if [[ "${1:-}" == "--check" ]]; then
    echo "bootstrap-profile check:"
    local env_present="missing"
    [[ -f "$ENV_FILE" ]] && env_present="present"
    local docker_state="missing"
    command -v docker >/dev/null 2>&1 && docker_state="ok"
    local cargo_state="missing"
    command -v cargo >/dev/null 2>&1 && cargo_state="ok"
    local profile_count
    profile_count="$(find "$PROFILES_DIR" -maxdepth 1 -name '*.env' 2>/dev/null | wc -l | tr -d ' ')"
    echo "  .env      : $env_present"
    echo "  docker    : $docker_state"
    echo "  cargo     : $cargo_state"
    echo "  profiles  : $profile_count available"
    exit 0
  fi
  if [[ "${1:-}" == "--wizard" ]]; then
    select_profile_wizard; exit 0
  fi
  local profile="${1:-quick-start}"
  apply_profile "${profile}"
}

main "$@"

