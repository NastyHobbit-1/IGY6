# Codex Prompt: DIFF-130 Ollama Setup And Task-Based Model Routing

Use this prompt after DIFF-129 to add safe local Ollama installation/model bootstrap support plus task-based local model routing.

```text
You are working in the IGY6 repo.

Mission:
Create the next available DIFF to add safe local Ollama setup plus task-based local model routing for IGY6.

The system should check/install Ollama only when explicitly requested, pull only the best-fit local models for this project and hardware, and route different task types to different local models with different system instructions.

Important user authorization:
The repo owner explicitly wants Codex to make this ready to go. If this Codex session is running on the user's actual local IGY6 machine, and not in an isolated cloud/Codespaces environment, Codex may run the newly added setup script after verification to install Ollama if missing and pull the approved default models. If running in Codespaces, CI, or any environment that is not the user's local WSL/Linux machine, do not install or pull models; create the script/docs/config only and report that local execution is required.

Do not make external model calls.
Do not add cloud providers.
Do not require API keys.
Do not commit secrets.
Do not make Ollama mandatory for normal IGY6 startup.
Do not remove deterministic evidence fallback.
Do not claim Rust-only.
Do not run destructive commands.
Do not install anything unless the command is explicitly scoped and safe.
Do not pull large models by default.
Do not pull every model.

Before editing:
- Inspect git status.
- Inspect git log --oneline -n 30.
- Read AGENTS.md.
- Read docs/diffs/.
- Read docs/agents/.
- Read README.md.
- Read docs/user-guide.md.
- Read docs/llm/LOCAL_LLM_PROVIDER_PLAN.md.
- Read .env.example.
- Read infra/docker-compose.yml.
- Read scripts/runtime-smoke.sh if present.
- Read scripts/e2e-manual-upload-smoke.* if present.
- Inspect crates/igy6-llm/ if present.
- Inspect crates/igy6-gateway/ LLM settings/status and evidence answer code.
- Determine the next correct DIFF number from the repo itself.

Expected next DIFF:
DIFF-130 unless the repo already contains a newer active or locked DIFF.

Goal:
Add a safe Ollama local setup helper and a task-based model routing plan/config for IGY6.

Hardware target:
- Primary target is a normal local PC with NVIDIA RTX 3060 12GB VRAM.
- Prefer models that fit practical 12GB VRAM use.
- Avoid huge default pulls that cause slow CPU/RAM offload.

Default model set:
Pull only these by default when model pulling is explicitly requested:
- qwen2.5-coder:7b
- llama3.1:8b
- gemma3:4b

Optional model:
- gemma3:12b

Do not pull by default:
- qwen2.5-coder:32b
- llama3.1:70b
- llama3.1:405b
- gemma3:27b
- any other large model unless the user explicitly edits the config later

Task routing requirements:
Add a local model routing config/documentation that maps task types to model, system instruction, temperature, and purpose.

Required task routes:

1. code_repo
- Model: qwen2.5-coder:7b
- Purpose: code, logs, stack traces, DIFF prompts, repo summaries, route parity, scripts
- System instruction:
  You are IGY6 local code assistant. Use only provided repo context and retrieved evidence. Be precise, cite files/routes when available, do not invent repo state, do not suggest unsafe commands, and preserve DIFF governance.

2. evidence_summary
- Model: llama3.1:8b
- Purpose: summarize uploaded notes, warranty text, bills, router notes, general evidence records
- System instruction:
  You are IGY6 evidence summarizer. Answer only from retrieved evidence. Include uncertainty and missing information. Do not invent facts. If evidence is insufficient, say insufficient evidence.

3. fast_triage
- Model: gemma3:4b
- Purpose: quick classification, short summaries, first-pass issue triage, UI helper explanations
- System instruction:
  You are IGY6 fast local triage. Give concise, evidence-grounded output. Prefer short actionable summaries. Do not make external calls. Do not execute actions.

4. report_draft
- Model: llama3.1:8b by default
- Optional model: gemma3:12b if installed and explicitly selected
- Purpose: longer evidence-grounded reports and migration summaries
- System instruction:
  You are IGY6 report drafter. Produce structured reports from retrieved evidence only. Preserve citations/source trails. Separate facts, assumptions, uncertainty, and next actions.

5. action_explanation
- Model: gemma3:4b
- Purpose: explain approval-required actions, safety reasons, audit summaries
- System instruction:
  You are IGY6 safety explainer. Explain what an action would do, why approval may be required, and what audit records should exist. Do not approve, execute, or bypass policy.

6. chat_default
- Model: llama3.1:8b
- Purpose: default Assistant evidence-grounded chat when task cannot be classified as code-specific
- System instruction:
  You are IGY6 Assistant. Use retrieved evidence only. If the answer is not supported, say insufficient evidence. Do not execute actions. Do not reveal secrets. Keep local-first behavior.

Routing behavior:
- Add config for task routes, for example configs/local-llm-routing.json.
- The config must include:
  - task_name
  - model
  - optional_model if applicable
  - purpose
  - system_instruction
  - temperature
  - evidence_required
  - max_context_note or budget note
- Default all tasks to evidence_required=true.
- Default provider must remain none unless explicitly enabled.
- Do not make real model calls during tests.
- Add validation for routing config if practical.
- If Rust LLM crate already has provider abstractions, add route selection structures/tests there.
- If implementation is too large, add config + docs + script first and leave full runtime router wiring for the next DIFF, but do not fake completion.

Required script:
Add or update:

scripts/ollama-local-setup.sh

Script requirements:
- Bash/WSL compatible.
- Use set -Eeuo pipefail.
- Default mode must be check-only.
- Must not install Ollama unless passed:
  --install --yes
- Must not pull models unless passed:
  --pull-default-models
  or --pull-model MODEL
- Must not change .env unless passed:
  --write-env MODEL_OR_TASK
- Must not overwrite .env without creating a backup.
- Must not require sudo unless --install is used and Ollama installer requires it.
- Must print clear PASS/FAIL/NEXT lines.
- Must detect whether ollama command exists.
- Must detect whether Ollama API is reachable at:
  http://127.0.0.1:11434
- Must list installed models if ollama is available.
- Must recommend task models based on the routing config.
- Must support:
  --check
  --install --yes
  --pull-default-models
  --pull-model MODEL
  --write-env MODEL_OR_TASK
  --list-recommended
  --help
- Must avoid down -v or destructive Docker commands.
- Must not delete Ollama models.
- Must not expose secrets.
- Must not call cloud APIs.
- Must not install or pull anything during normal verification.

Install behavior:
- If --install --yes is used and ollama is missing, use the official Linux installer:
  curl -fsSL https://ollama.com/install.sh | sh
- If --install is used without --yes, print what would happen and exit safely.
- Do not install automatically during tests.

Model pull behavior:
- --pull-default-models pulls only:
  - qwen2.5-coder:7b
  - llama3.1:8b
  - gemma3:4b
- --pull-model MODEL may pull one explicitly named model.
- Script should warn before pulling gemma3:12b that it is heavier and optional.
- Script should warn against 32B/70B/405B models on 12GB VRAM default setup.

.env behavior:
If --write-env is used:
- Backup .env first.
- Set:
  LLM_PROVIDER=ollama
  OLLAMA_BASE_URL=http://host.docker.internal:11434
  OLLAMA_MODEL=<default model or selected model>
  LLM_TIMEOUT_SECONDS=60
  LLM_EVIDENCE_REQUIRED=true
- Do not write secrets.
- Do not remove existing unrelated .env values.
- If writing for a task route, use that task's model as OLLAMA_MODEL.
- Document that runtime task routing may still select task-specific models when enabled.

Codex local execution requirement:
After DIFF-130 implementation, verification, commit, and push, if and only if this is the user's actual local WSL/Linux environment:
1. Run scripts/ollama-local-setup.sh --check.
2. If Ollama is missing, run scripts/ollama-local-setup.sh --install --yes.
3. Run scripts/ollama-local-setup.sh --pull-default-models.
4. Run scripts/ollama-local-setup.sh --write-env qwen2.5-coder:7b.
5. Run scripts/ollama-local-setup.sh --check again.
6. Report installed/pulled models and whether the Ollama API is reachable.

If this is Codespaces/CI/remote sandbox, do not install or pull; report that the script is ready and must be run locally.

Documentation updates:
Update:
- README.md
- docs/user-guide.md
- docs/llm/LOCAL_LLM_PROVIDER_PLAN.md
- Any relevant runtime smoke docs

Docs must explain:
- Ollama is optional.
- IGY6 still works without Ollama using deterministic evidence fallback.
- No external model calls by default.
- Ollama must be local.
- Only the recommended models should be pulled by default.
- Why qwen2.5-coder:7b is the default for code/repo work.
- Why llama3.1:8b is the default for general evidence summaries.
- Why gemma3:4b is the fast fallback.
- Why gemma3:12b is optional and heavier.
- How task-based routing works.
- How each task uses a different system instruction.
- How to check if Ollama is installed:
  ollama --version
- How to check if Ollama is running:
  curl http://127.0.0.1:11434/api/tags
- How to install Ollama manually:
  curl -fsSL https://ollama.com/install.sh | sh
- How to pull default models:
  ollama pull qwen2.5-coder:7b
  ollama pull llama3.1:8b
  ollama pull gemma3:4b
- How to test model manually:
  ollama run qwen2.5-coder:7b
- How to configure IGY6:
  LLM_PROVIDER=ollama
  OLLAMA_BASE_URL=http://host.docker.internal:11434
  OLLAMA_MODEL=qwen2.5-coder:7b
  LLM_EVIDENCE_REQUIRED=true
- How to revert to deterministic mode:
  LLM_PROVIDER=none

Docs must include model recommendation table:
- qwen2.5-coder:7b as code/repo/default model
- llama3.1:8b for general evidence summaries and default chat
- gemma3:4b for fast/lightweight triage
- gemma3:12b for optional longer report drafting if performance is acceptable

Safety requirements:
- Follow AGENTS.md exactly.
- Follow DIFF process exactly.
- Only one active DIFF at a time.
- Locked DIFFs must not be edited.
- No unrelated refactor.
- No broad cleanup.
- No secrets.
- No .env commit.
- No runtime/private data commit.
- No automatic install during tests.
- No automatic model pull during tests.
- No cloud providers.
- No external model calls.
- No destructive commands.
- No deletion of Ollama models.
- No Rust-only claim.
- No FastAPI removal.
- No LLM action execution.
- No action approval bypass.

Verification:
- git status --short
- git diff --check
- bash -n scripts/ollama-local-setup.sh
- scripts/ollama-local-setup.sh --help
- scripts/ollama-local-setup.sh --check
- scripts/ollama-local-setup.sh --list-recommended
- python3 -m json.tool configs/local-llm-routing.json if created
- cargo fmt --all --check if Rust files changed
- cargo clippy --workspace --all-targets if Rust files changed
- cargo test --workspace if Rust files changed
- npm --prefix apps/web run build if docs/UI touched in a way that affects web
- npm --prefix apps/web run test:ui-smoke if UI copy changed
- npm --prefix apps/web test if package scripts exist
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- docker compose -f infra/docker-compose.yml --env-file .env.example config

Commit and push:
- Commit with DIFF-130 in the message.
- Push to origin/main after verification passes.
- If push fails, report exact error.

Final response must include:
- DIFF number
- Ollama setup script added/updated
- whether install is check-only by default
- install command supported
- whether Codex installed Ollama locally or skipped because environment was not local
- default model set
- which models were actually pulled, if local execution happened
- optional model
- task routing config added
- task-to-model mapping
- task-specific system instructions added
- .env write behavior
- safety behavior
- docs updated
- verification results
- commit hash
- push status
- next recommended DIFF
```
