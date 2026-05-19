#!/usr/bin/env bash
set -Eeuo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ROUTING_CONFIG="${REPO_ROOT}/configs/local-llm-routing.json"
ENV_FILE="${IGY6_ENV_FILE:-${REPO_ROOT}/.env}"
OLLAMA_API_URL="${OLLAMA_API_URL:-http://127.0.0.1:11434}"

MODE_CHECK=1
INSTALL=0
YES=0
PULL_DEFAULTS=0
LIST_RECOMMENDED=0
WRITE_ENV_TARGET=""
PULL_MODELS=()

DEFAULT_MODELS=(
  "qwen2.5-coder:7b"
  "llama3.1:8b"
  "gemma3:4b"
)
OPTIONAL_MODELS=(
  "gemma3:12b"
)
LARGE_MODEL_PATTERNS=(
  "32b"
  "70b"
  "405b"
  "27b"
)

usage() {
  cat <<EOF
IGY6 local Ollama setup helper

Usage:
  scripts/ollama-local-setup.sh [--check] [--list-recommended]
  scripts/ollama-local-setup.sh --install --yes
  scripts/ollama-local-setup.sh --pull-default-models
  scripts/ollama-local-setup.sh --pull-model MODEL
  scripts/ollama-local-setup.sh --write-env MODEL_OR_TASK

Default:
  --check   Inspect local Ollama command/API/model state only. No install, pull,
            Docker change, model deletion, cloud call, or .env write.

Options:
  --check                 Check command availability, local API, installed models, and routing config.
  --install               Show install intent. Requires --yes to run the official Linux installer.
  --yes                   Confirm --install.
  --pull-default-models   Pull only qwen2.5-coder:7b, llama3.1:8b, and gemma3:4b.
  --pull-model MODEL      Pull one explicitly named model.
  --write-env MODEL_OR_TASK
                          Backup .env and set local Ollama env keys. MODEL_OR_TASK may be
                          a model tag or a task from configs/local-llm-routing.json.
  --list-recommended      Print task-to-model routing recommendations.
  --help                  Show this help.

Safety:
  Ollama is optional. IGY6 keeps deterministic evidence fallback when
  LLM_PROVIDER=none or the local provider is unavailable.
EOF
}

pass() {
  printf 'PASS %s\n' "$1"
}

fail() {
  printf 'FAIL %s\n' "$1"
}

next() {
  printf 'NEXT %s\n' "$1"
}

warn() {
  printf 'WARN %s\n' "$1"
}

die() {
  fail "$1"
  exit 1
}

have_command() {
  command -v "$1" >/dev/null 2>&1
}

http_status() {
  curl --silent --show-error --output /dev/null --write-out '%{http_code}' --max-time 5 "$1" 2>/dev/null || true
}

validate_routing_config() {
  if [[ ! -f "${ROUTING_CONFIG}" ]]; then
    die "routing config not found: ${ROUTING_CONFIG}"
  fi
  if python3 -m json.tool "${ROUTING_CONFIG}" >/dev/null; then
    pass "routing config JSON is valid"
  else
    die "routing config JSON is invalid"
  fi
}

list_recommended() {
  validate_routing_config
  python3 - "$ROUTING_CONFIG" <<'PY'
import json
import sys

path = sys.argv[1]
with open(path, "r", encoding="utf-8") as handle:
    config = json.load(handle)

print("Recommended default pulls:")
for model in config["default_models"]:
    print(f"  - {model}")
print("Optional models:")
for model in config.get("optional_models", []):
    print(f"  - {model}")
print("Task routing:")
for task in config["tasks"]:
    optional = f" optional={task['optional_model']}" if task.get("optional_model") else ""
    print(f"  - {task['task_name']}: model={task['model']}{optional} temperature={task['temperature']}")
    print(f"    purpose={task['purpose']}")
PY
}

check_ollama() {
  validate_routing_config
  if have_command ollama; then
    pass "ollama command is available"
    ollama --version 2>/dev/null || true
  else
    fail "ollama command is not available"
    next "Install locally with: scripts/ollama-local-setup.sh --install --yes"
  fi

  if have_command curl; then
    pass "curl is available"
    local status
    status="$(http_status "${OLLAMA_API_URL}/api/tags")"
    case "${status}" in
      2*)
        pass "Ollama API reachable at ${OLLAMA_API_URL}"
        ;;
      000|"")
        fail "Ollama API is not reachable at ${OLLAMA_API_URL}"
        next "Start Ollama locally, then retry --check."
        ;;
      *)
        fail "Ollama API returned HTTP ${status} at ${OLLAMA_API_URL}"
        ;;
    esac
  else
    fail "curl is not available"
  fi

  if have_command ollama; then
    printf 'Installed Ollama models:\n'
    ollama list 2>/dev/null || next "Run ollama list after starting Ollama."
  fi
  list_recommended
}

install_ollama() {
  if have_command ollama; then
    pass "ollama already installed"
    return
  fi
  if [[ "${YES}" -ne 1 ]]; then
    next "Would install Ollama with: curl -fsSL https://ollama.com/install.sh | sh"
    next "Re-run with --install --yes to perform the install."
    exit 0
  fi
  pass "installing Ollama with official Linux installer"
  curl -fsSL https://ollama.com/install.sh | sh
}

is_large_model() {
  local model="$1"
  local pattern
  for pattern in "${LARGE_MODEL_PATTERNS[@]}"; do
    if [[ "${model,,}" == *"${pattern}"* ]]; then
      return 0
    fi
  done
  return 1
}

warn_model_size() {
  local model="$1"
  if [[ "${model}" == "gemma3:12b" ]]; then
    warn "gemma3:12b is optional and heavier; use only if local performance is acceptable."
  fi
  if is_large_model "${model}"; then
    warn "${model} is not recommended as a default pull for RTX 3060 12GB VRAM."
  fi
}

pull_model() {
  local model="$1"
  have_command ollama || die "ollama command is required before pulling models"
  warn_model_size "${model}"
  pass "pulling model ${model}"
  ollama pull "${model}"
}

pull_default_models() {
  local model
  for model in "${DEFAULT_MODELS[@]}"; do
    pull_model "${model}"
  done
}

model_for_task_or_model() {
  local target="$1"
  python3 - "$ROUTING_CONFIG" "$target" <<'PY'
import json
import sys

path, target = sys.argv[1], sys.argv[2]
with open(path, "r", encoding="utf-8") as handle:
    config = json.load(handle)
for task in config["tasks"]:
    if task["task_name"] == target:
        print(task["model"])
        raise SystemExit(0)
print(target)
PY
}

set_env_value() {
  local file="$1"
  local key="$2"
  local value="$3"
  local tmp="${file}.tmp.$$"
  if [[ -f "${file}" ]] && grep -q "^${key}=" "${file}"; then
    awk -v key="${key}" -v value="${value}" '
      BEGIN { done=0 }
      $0 ~ "^" key "=" {
        print key "=" value
        done=1
        next
      }
      { print }
      END {
        if (done == 0) print key "=" value
      }
    ' "${file}" > "${tmp}"
  else
    if [[ -f "${file}" ]]; then
      cp "${file}" "${tmp}"
      printf '%s=%s\n' "${key}" "${value}" >> "${tmp}"
    else
      printf '%s=%s\n' "${key}" "${value}" > "${tmp}"
    fi
  fi
  mv "${tmp}" "${file}"
}

write_env() {
  local target="$1"
  validate_routing_config
  local model
  model="$(model_for_task_or_model "${target}")"
  [[ -n "${model}" ]] || die "could not resolve model for ${target}"

  if [[ ! -f "${ENV_FILE}" ]]; then
    if [[ -f "${REPO_ROOT}/.env.example" ]]; then
      cp "${REPO_ROOT}/.env.example" "${ENV_FILE}"
      pass "created .env from .env.example"
    else
      die ".env is missing and .env.example is not available"
    fi
  fi

  local backup="${ENV_FILE}.bak.$(date -u +%Y%m%d%H%M%S)"
  cp "${ENV_FILE}" "${backup}"
  pass "backed up .env to ${backup}"

  set_env_value "${ENV_FILE}" "LLM_PROVIDER" "ollama"
  set_env_value "${ENV_FILE}" "OLLAMA_BASE_URL" "http://host.docker.internal:11434"
  set_env_value "${ENV_FILE}" "OLLAMA_MODEL" "${model}"
  set_env_value "${ENV_FILE}" "LLM_TIMEOUT_SECONDS" "60"
  set_env_value "${ENV_FILE}" "LLM_EVIDENCE_REQUIRED" "true"

  pass "updated local Ollama env keys in ${ENV_FILE}"
  next "Restart/recreate the IGY6 stack for .env changes to reach containers."
  next "Runtime task routing may still select task-specific models when enabled."
}

while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --check)
      MODE_CHECK=1
      ;;
    --install)
      MODE_CHECK=0
      INSTALL=1
      ;;
    --yes)
      YES=1
      ;;
    --pull-default-models)
      MODE_CHECK=0
      PULL_DEFAULTS=1
      ;;
    --pull-model)
      MODE_CHECK=0
      shift
      [[ "$#" -gt 0 ]] || die "--pull-model requires MODEL"
      PULL_MODELS+=("$1")
      ;;
    --write-env)
      MODE_CHECK=0
      shift
      [[ "$#" -gt 0 ]] || die "--write-env requires MODEL_OR_TASK"
      WRITE_ENV_TARGET="$1"
      ;;
    --list-recommended)
      MODE_CHECK=0
      LIST_RECOMMENDED=1
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
  shift
done

if [[ "${MODE_CHECK}" -eq 1 ]]; then
  check_ollama
fi
if [[ "${LIST_RECOMMENDED}" -eq 1 ]]; then
  list_recommended
fi
if [[ "${INSTALL}" -eq 1 ]]; then
  install_ollama
fi
if [[ "${PULL_DEFAULTS}" -eq 1 ]]; then
  pull_default_models
fi
for model in "${PULL_MODELS[@]}"; do
  pull_model "${model}"
done
if [[ -n "${WRITE_ENV_TARGET}" ]]; then
  write_env "${WRITE_ENV_TARGET}"
fi
