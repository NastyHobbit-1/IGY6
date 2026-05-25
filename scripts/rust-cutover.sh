#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MANIFEST="${REPO_ROOT}/configs/rust-cutover-manifest.json"
MODE="check"

usage() {
  cat <<'EOF'
Usage:
  scripts/rust-cutover.sh [--check|--plan|--dry-run|--execute|--help]

Modes:
  --check     Validate readiness only. Default.
  --plan      Print planned archive/rewrite/create actions.
  --dry-run   Show what would happen without changing files.
  --execute   Perform final Rust cutover. Requires clean git and manifest cutover_ready=true.
  --help      Show this help.

Safety:
  - Default mode is check-only.
  - Does not delete files.
  - Does not move runtime/private data.
  - Does not modify .env.
  - Refuses execute unless manifest cutover_ready is true.
  - Refuses execute unless git worktree is clean.
EOF
}

die() {
  printf 'ERROR: %s\n' "$*" >&2
  exit 1
}

warn() {
  printf 'WARN: %s\n' "$*" >&2
}

info() {
  printf '%s\n' "$*"
}

for arg in "$@"; do
  case "${arg}" in
    --check) MODE="check" ;;
    --plan) MODE="plan" ;;
    --dry-run) MODE="dry-run" ;;
    --execute) MODE="execute" ;;
    --help|-h) usage; exit 0 ;;
    *) die "Unknown argument: ${arg}" ;;
  esac
done

require_file() {
  local path="$1"
  [[ -f "${path}" ]] || die "Missing required file: ${path}"
}

require_command() {
  local command_name="$1"
  command -v "${command_name}" >/dev/null 2>&1 || die "Missing required command: ${command_name}"
}

json_value() {
  local expression="$1"
  python3 - "${MANIFEST}" "${expression}" <<'PY'
import json
import sys

manifest_path = sys.argv[1]
expression = sys.argv[2]
with open(manifest_path, encoding="utf-8") as handle:
    value = json.load(handle)

for part in expression.split("."):
    if not part:
        continue
    if isinstance(value, dict):
        value = value.get(part)
    else:
        value = None
        break

if isinstance(value, bool):
    print("true" if value else "false")
elif value is None:
    print("")
else:
    print(value)
PY
}

validate_manifest() {
  python3 - "${MANIFEST}" <<'PY'
import json
import sys

manifest_path = sys.argv[1]
with open(manifest_path, encoding="utf-8") as handle:
    data = json.load(handle)

required_top = ["schema_version", "cutover_ready", "target_architecture", "required_phases", "phases", "archive_plan"]
missing = [key for key in required_top if key not in data]
if missing:
    raise SystemExit(f"missing manifest keys: {missing}")

valid_targets = {"rust-primary", "rust-only-application-runtime"}
if data["target_architecture"] not in valid_targets:
    raise SystemExit("target_architecture must be rust-primary or rust-only-application-runtime")

if not isinstance(data["cutover_ready"], bool):
    raise SystemExit("cutover_ready must be boolean")

required_phases = data["required_phases"]
phases = data["phases"]
missing_phases = [phase for phase in required_phases if phase not in phases]
if missing_phases:
    raise SystemExit(f"missing phase entries: {missing_phases}")

valid_statuses = {"pending", "partial", "complete"}
for phase, entry in phases.items():
    status = entry.get("status")
    if status not in valid_statuses:
        raise SystemExit(f"phase {phase} has invalid status {status!r}")

archive_plan = data["archive_plan"]
for key in ("move", "keep", "rewrite", "create_if_missing"):
    if key not in archive_plan or not isinstance(archive_plan[key], list):
        raise SystemExit(f"archive_plan.{key} must be a list")
PY
}

require_clean_git() {
  local status_output
  status_output="$(git -C "${REPO_ROOT}" status --short)"
  [[ -z "${status_output}" ]] || die "Git worktree is not clean. Commit or clear changes before final cutover."
}

run_cargo_checks_if_present() {
  if [[ ! -f "${REPO_ROOT}/Cargo.toml" ]]; then
    warn "Cargo.toml does not exist yet. Rust workspace phase is not complete."
    return
  fi

  require_command cargo
  info "Running Rust workspace checks..."
  cargo test --manifest-path "${REPO_ROOT}/Cargo.toml" --workspace
  cargo fmt --all --check
  cargo clippy --manifest-path "${REPO_ROOT}/Cargo.toml" --workspace --all-targets
}

run_check() {
  info "Checking Rust cutover readiness..."
  require_command python3
  require_command git
  require_file "${MANIFEST}"
    require_file "${REPO_ROOT}/infra/docker-compose.yml"

  validate_manifest
  run_cargo_checks_if_present
  if [[ -f "${REPO_ROOT}/scripts/rust-route-parity.py" ]]; then
    info "Running Rust/FastAPI route parity guard..."
    python3 "${REPO_ROOT}/scripts/rust-route-parity.py" --check
  fi

  local ready
  ready="$(json_value "cutover_ready")"
  if [[ "${ready}" != "true" ]]; then
    warn "Manifest cutover_ready is false. Final cutover is blocked."
  fi

  info "Check complete."
}

print_plan() {
  require_command python3
  require_file "${MANIFEST}"
  validate_manifest
  python3 - "${MANIFEST}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)

print("Rust cutover manifest")
print(f"  schema_version: {data.get('schema_version')}")
print(f"  cutover_ready: {data.get('cutover_ready')}")
print(f"  target_architecture: {data.get('target_architecture')}")

print("\nRequired phases")
for phase in data.get("required_phases", []):
    entry = data.get("phases", {}).get(phase, {})
    print(f"  {phase}: {entry.get('status')}")

plan = data.get("archive_plan", {})
for section in ("move", "keep", "rewrite", "create_if_missing"):
    print(f"\n[{section}]")
    items = plan.get(section, [])
    if not items:
        print("  none")
    for item in items:
        print(f"  {json.dumps(item, sort_keys=True)}")
PY
}

assert_safe_relative_path() {
  local path="$1"
  [[ -n "${path}" ]] || die "Archive path is empty"
  [[ "${path}" != /* ]] || die "Archive path must be relative: ${path}"
  [[ "${path}" != *".."* ]] || die "Archive path must not contain '..': ${path}"
  case "${path}" in
    .env|.env.*|storage|storage/*|IGY6_Data|IGY6_Data/*)
      die "Archive plan must not touch runtime/private path: ${path}"
      ;;
  esac
}

plan_entries() {
  local section="$1"
  python3 - "${MANIFEST}" "${section}" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)

for item in data.get("archive_plan", {}).get(sys.argv[2], []):
    print(json.dumps(item, sort_keys=True))
PY
}

dry_run_or_execute_moves() {
  while IFS= read -r item_json; do
    [[ -n "${item_json}" ]] || continue
    local source
    local target
    source="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1]).get("from", ""))' "${item_json}")"
    target="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1]).get("to", ""))' "${item_json}")"
    assert_safe_relative_path "${source}"
    assert_safe_relative_path "${target}"
    [[ "${target}" == archive/* ]] || die "Archive move target must be under archive/: ${target}"

    if [[ ! -e "${REPO_ROOT}/${source}" ]]; then
      warn "Archive source does not exist, skipping: ${source}"
      continue
    fi

    if [[ "${MODE}" == "dry-run" ]]; then
      info "DRY-RUN git mv ${source} ${target}"
    else
      mkdir -p "$(dirname "${REPO_ROOT}/${target}")"
      info "git mv ${source} ${target}"
      git -C "${REPO_ROOT}" mv "${source}" "${target}"
    fi
  done < <(plan_entries move)
}

dry_run_or_execute_creates() {
  while IFS= read -r item_json; do
    [[ -n "${item_json}" ]] || continue
    local path
    local title
    path="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1]).get("path", ""))' "${item_json}")"
    title="$(python3 -c 'import json,sys; print(json.loads(sys.argv[1]).get("title", "IGY6 Rust"))' "${item_json}")"
    assert_safe_relative_path "${path}"

    if [[ -e "${REPO_ROOT}/${path}" ]]; then
      info "Exists, not creating: ${path}"
      continue
    fi

    if [[ "${MODE}" == "dry-run" ]]; then
      info "DRY-RUN create ${path}"
    else
      mkdir -p "$(dirname "${REPO_ROOT}/${path}")"
      cat > "${REPO_ROOT}/${path}" <<EOF
# ${title}

This file was created during the Rust cutover.

Update this document with the current Rust architecture, operations, and usage details.
EOF
      git -C "${REPO_ROOT}" add "${path}"
    fi
  done < <(plan_entries create_if_missing)
}

execute_manifest_plan() {
  info "Applying manifest archive/create plan in ${MODE} mode..."
  dry_run_or_execute_moves
  dry_run_or_execute_creates
  info "Keeping docs/diffs/ active as locked project history. Build-agent instructions are dev-only and not active on main."
}

execute_cutover() {
  info "Starting final Rust cutover..."
  require_clean_git
  run_check

  local ready
  ready="$(json_value "cutover_ready")"
  [[ "${ready}" == "true" ]] || die "Manifest cutover_ready is not true. Refusing final cutover."

  MODE="execute"
  execute_manifest_plan
  info "Final cutover plan applied. Review git status and run final verification before committing."
}

cd "${REPO_ROOT}"

case "${MODE}" in
  check)
    run_check
    ;;
  plan)
    print_plan
    ;;
  dry-run)
    run_check
    print_plan
    execute_manifest_plan
    ;;
  execute)
    execute_cutover
    ;;
  *)
    die "Unhandled mode: ${MODE}"
    ;;
esac
