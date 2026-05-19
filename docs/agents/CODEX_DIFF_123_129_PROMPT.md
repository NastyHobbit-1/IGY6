# Codex Prompt: DIFF-123 through DIFF-129

Use this prompt to continue IGY6 from DIFF-122 through the runtime smoke, end-to-end evidence pipeline, worker verification, and local LLM integration phases.

```text
You are working in the IGY6 repo.

Mission:
Continue from DIFF-122 and complete DIFF-123 through DIFF-129 in order, only stopping for a true hard blocker.

The goal is to move from “UI reorganized and smoke-tested” to “runtime smoke tested, end-to-end evidence pipeline tested, worker/processing verified, local LLM planned, local LLM adapter added, evidence-grounded LLM answers wired into Assistant, and UI controls added for model/provider status.”

Do not stop after one DIFF unless the DIFF process requires a commit boundary. If the repo requires one active DIFF at a time, complete one DIFF fully, verify it, lock it, commit it, push it, then immediately continue to the next DIFF.

Only stop for a true hard blocker.

A true hard blocker means:
- A required command fails and cannot be fixed safely inside the active DIFF.
- Continuing would violate AGENTS.md.
- Continuing would edit locked DIFFs.
- Continuing would require secrets, external API keys, private credentials, or unapproved runtime/private data.
- Continuing would require destructive data loss.
- Continuing would require a product decision that cannot be inferred from repo docs.

Do not stop for:
- Needing another DIFF.
- Test failures that can be fixed.
- Formatting/clippy/build failures that can be fixed.
- Missing docs that can be written.
- Missing scripts that can be added.
- Missing UI helper text.
- Need to update README/user guide.
- Need to inspect repo files.
- Need to adjust route parity docs honestly.
- Need to keep FastAPI fallback documented.
- Ollama not running locally, as long as the code can support configured local Ollama and tests can mock/validate behavior.

Before editing:
- Inspect git status.
- Inspect git log --oneline -n 30.
- Read AGENTS.md.
- Read docs/diffs/.
- Read docs/agents/.
- Read README.md.
- Read docs/user-guide.md.
- Read apps/web/package.json.
- Read apps/web/scripts/ui-smoke.mjs.
- Read apps/web/src/app/page.tsx.
- Read apps/web/src/app/layout.tsx.
- Read infra/docker-compose.yml.
- Read .env.example.
- Read configs/rust-cutover-manifest.json.
- Read configs/legacy-fastapi-route-classification.json if present.
- Read docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md.
- Read docs/rust-migration/NON_WEB_FASTAPI_ROUTE_CLASSIFICATION.md if present.
- Read scripts/rust-route-parity.py.
- Determine the latest locked DIFF and the next valid DIFF number from the repo itself.

Expected sequence:
- DIFF-123: Runtime smoke test docs/script
- DIFF-124: End-to-end manual upload -> evidence -> chat retrieval test
- DIFF-125: Worker/processing verification and bug fixes
- DIFF-126: Local LLM provider integration plan
- DIFF-127: Local LLM adapter, starting with Ollama
- DIFF-128: Wire LLM into Assistant answer generation with evidence-required mode
- DIFF-129: UI controls for model/provider/status

If the repo already has any of these DIFFs, continue from the next correct DIFF.

Global safety rules:
- Follow AGENTS.md exactly.
- Follow the DIFF process exactly.
- Only one active DIFF at a time.
- Locked DIFFs must not be edited.
- Every change must be inside the active DIFF scope.
- No unrelated refactor.
- No broad cleanup.
- No .env commits.
- No secrets.
- No private keys.
- No tokens.
- No runtime/private data commits.
- No unsafe deletion.
- No Docker volume deletion.
- No down -v in normal docs/scripts.
- No arbitrary shell execution.
- No user-provided argv execution.
- No approval bypass.
- No unapproved external model/API calls.
- No claiming Rust-only unless manifest, route parity, Compose, and docs prove it.
- No hiding FastAPI fallback status if it remains.
- No LLM answer without evidence unless explicitly labeled insufficient evidence or general system help.

Persistence rule:
After each DIFF:
- Run verification.
- Lock the DIFF only after verification passes.
- Commit with the DIFF number in the commit message.
- Push to origin/main.
- Continue to the next DIFF until DIFF-129 is complete or a true hard blocker occurs.

DIFF-123: Runtime smoke test docs/script

Goal:
Add practical runtime smoke-test instructions and a safe script for checking the local Docker stack.

Required work:
- Add scripts/runtime-smoke.sh or equivalent.
- Script must be Bash/WSL compatible.
- Use set -Eeuo pipefail.
- Default mode must check an already-running stack only.
- Must not start or stop unless explicit flags are passed.
- Must not use down -v.
- Must not delete files.
- Must not commit runtime/private data.
- Must check:
  - docker compose config is valid.
  - expected services are running when stack is up.
  - http://127.0.0.1:8000/health/live responds.
  - http://127.0.0.1:8000/health/ready responds if available.
  - http://127.0.0.1:3000 responds.
- Must print clear PASS/FAIL lines.
- On failure, print next diagnostic commands for web/api logs.
- Optional flags:
  - --check
  - --start
  - --stop
  - --detached
  - --help
- If --start is implemented, it must be explicit and safe.
- If --stop is implemented, it must use docker compose down, not down -v.
- Update README.md and docs/user-guide.md with runtime smoke test instructions.
- Explain:
  - how to start
  - how to stop
  - how to check ps
  - what empty ps means
  - what 127.0.0.1:3000 refused means
  - what Phoenix GET / 200 OK logs mean
  - long Docker commands
  - WSL aliases igy6-start, igy6-stop, igy6-ps, igy6-logs
- Create docs/diffs/DIFF-123-*.md.

DIFF-123 verification:
- git status --short
- git diff --check
- bash -n scripts/runtime-smoke.sh
- scripts/runtime-smoke.sh --help
- scripts/runtime-smoke.sh --check, allowed to fail clearly if stack is not running and must not create side effects
- npm --prefix apps/web run build
- npm --prefix apps/web run test:ui-smoke
- npm --prefix apps/web test
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- docker compose -f infra/docker-compose.yml --env-file .env.example config

DIFF-124: End-to-end manual upload -> evidence -> chat retrieval test

Goal:
Add a guided end-to-end test plan and, if practical, a safe automated or semi-automated smoke script for the core user path:
manual upload -> artifact -> work item -> processing -> evidence/chunks -> retrieval/chat.

Required work:
- Add docs/runtime/E2E_MANUAL_UPLOAD_SMOKE.md or similar.
- Add script if practical:
  - scripts/e2e-manual-upload-smoke.sh or scripts/e2e-manual-upload-smoke.py.
- The script must be safe and local-only.
- It must not commit uploaded test data.
- It must use a clearly harmless test payload:
  - “IGY6 manual upload test. The secret test keyword is blue-raven-117.”
- It must explain whether it is fully automated or checklist-assisted.
- It must check:
  - API live/ready.
  - web responds.
  - source creation or source existence.
  - approval path if required.
  - manual upload route.
  - work item creation.
  - evidence/chunk availability if processing completed.
  - chat/retrieval route if available.
- If worker processing is not guaranteed, script/docs must distinguish:
  - upload route passed
  - artifact/work item created
  - worker processing pending
  - evidence not yet generated
- Add user-facing docs for exactly what to click in the UI.
- Add troubleshooting for:
  - approval required
  - work item stuck queued
  - no evidence yet
  - chat cannot find keyword
  - worker logs to inspect
- Create docs/diffs/DIFF-124-*.md.

DIFF-124 verification:
- git status --short
- git diff --check
- bash -n script if Bash script created
- python3 -m py_compile script if Python script created
- npm --prefix apps/web run build
- npm --prefix apps/web run test:ui-smoke
- npm --prefix apps/web test
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- docker compose -f infra/docker-compose.yml --env-file .env.example config

DIFF-125: Worker/processing verification and bug fixes

Goal:
Verify and document the processing pipeline after manual upload:
raw artifact -> normalized document -> chunks -> evidence -> vector memory -> graph memory if applicable.

Required work:
- Inspect services/worker and any Rust worker crates.
- Inspect current work item status transitions.
- Inspect how manual upload creates queued normalization metadata.
- Determine whether Celery worker still owns processing.
- Determine whether Rust worker owns any processing.
- Add or update tests/scripts/docs to verify processing behavior honestly.
- Add processing status diagnostics to README/user-guide if missing.
- Add script or test to check:
  - worker container is running.
  - Redis is reachable.
  - Postgres is reachable.
  - queued work items can be inspected.
  - processing status can be reported.
  - Qdrant collection status can be checked.
- If bugs are found and fixable inside DIFF-125, fix them.
- Do not rewrite the worker architecture broadly.
- Do not migrate all worker behavior unless already scoped safely.
- Do not claim processing completes if only queued metadata is created.
- Create docs/diffs/DIFF-125-*.md.

DIFF-125 verification:
- git status --short
- git diff --check
- cargo fmt --all --check if Rust changed
- cargo clippy --workspace --all-targets if Rust changed
- cargo test --workspace if Rust changed
- python3 tests or compile checks for Python scripts if added
- npm --prefix apps/web run build if docs/UI changed
- npm --prefix apps/web test if web changed
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- docker compose -f infra/docker-compose.yml --env-file .env.example config

DIFF-126: Local LLM provider integration plan

Goal:
Plan LLM integration before writing model-calling code.

Required work:
- Add docs/llm/LOCAL_LLM_PROVIDER_PLAN.md or similar.
- Add/update README/user guide section explaining:
  - LLM is optional/local-first.
  - Start with Ollama.
  - No external model calls by default.
  - Evidence-first behavior remains.
  - Deterministic evidence answer remains fallback.
  - LLM must cite retrieved evidence.
  - If no evidence exists, answer must say insufficient evidence.
- Define provider interface requirements:
  - provider name
  - base URL
  - model name
  - health check
  - generate endpoint
  - timeout
  - max context/evidence budget
  - redaction/safety behavior
  - no secrets in logs
- Define config/env keys, but do not require real secrets:
  - LLM_PROVIDER=none|ollama
  - OLLAMA_BASE_URL=http://host.docker.internal:11434 or other local URL
  - OLLAMA_MODEL
  - LLM_TIMEOUT_SECONDS
  - LLM_EVIDENCE_REQUIRED=true
- Update .env.example only with safe local defaults if scoped.
- Add manifest/docs updates honestly.
- Do not call Ollama yet unless this DIFF explicitly implements only a health-plan stub.
- Create docs/diffs/DIFF-126-*.md.

DIFF-126 verification:
- git status --short
- git diff --check
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- python3 -m json.tool configs/rust-cutover-manifest.json if changed
- docker compose -f infra/docker-compose.yml --env-file .env.example config if .env.example/Compose changed
- npm --prefix apps/web run build if docs/UI references changed

DIFF-127: Local LLM adapter, starting with Ollama

Goal:
Add a local LLM provider abstraction and Ollama adapter without wiring it into normal answer generation yet.

Required behavior:
- Provider must be disabled by default unless configured.
- No external providers in this DIFF.
- Only local Ollama provider.
- No secrets required.
- No external API calls.
- Health check must be safe.
- Generate call must be timeout-bound.
- Output must be structured.
- Errors must be explicit.
- Redact sensitive-looking input/output in logs.
- Do not log full prompts if they may contain private evidence.
- Add tests using mocks/fakes, not a real Ollama dependency.
- If Rust gateway owns Assistant, implement provider abstraction in Rust.
- If existing backend structure suggests another crate, use the repo’s established pattern.
- Update .env.example if not already done:
  - LLM_PROVIDER=none
  - OLLAMA_BASE_URL=http://host.docker.internal:11434
  - OLLAMA_MODEL=
  - LLM_TIMEOUT_SECONDS=60
  - LLM_EVIDENCE_REQUIRED=true
- Add local health/status route only if scoped and safe.
- Do not wire LLM into chat answer generation yet unless it remains disabled and test-only.
- Create docs/diffs/DIFF-127-*.md.

DIFF-127 verification:
- git status --short
- git diff --check
- cargo fmt --all --check
- cargo clippy --workspace --all-targets
- cargo test --workspace
- cargo test -p igy6-gateway if gateway changed
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- python3 -m json.tool configs/rust-cutover-manifest.json if changed
- docker compose -f infra/docker-compose.yml --env-file .env.example config
- npm --prefix apps/web run build if web changed

DIFF-128: Evidence-grounded LLM answer generation

Goal:
Wire the local LLM provider into Assistant answer generation while preserving evidence-required behavior.

Required behavior:
- Deterministic evidence answer must remain available as fallback.
- LLM must only answer using retrieved evidence when LLM_EVIDENCE_REQUIRED=true.
- If no evidence exists, return insufficient evidence.
- LLM prompt must include evidence packet/source trails, not raw private data beyond the retrieved evidence budget.
- LLM answer must include citations/source trails where possible.
- LLM must not invent unsupported claims.
- LLM must not execute actions.
- LLM must not call external services.
- LLM call must be timeout-bound.
- LLM disabled/missing provider must degrade gracefully to deterministic evidence answer.
- Add tests:
  - no evidence -> insufficient evidence
  - provider disabled -> deterministic fallback
  - provider timeout -> fallback/error response
  - evidence provided -> LLM adapter called with bounded evidence packet
  - citations/source trails preserved
  - sensitive-looking values redacted from logs
- Update Assistant UI copy only if needed:
  - “Evidence-grounded answer”
  - “Deterministic fallback”
  - “LLM unavailable”
  - “Insufficient evidence”
- Do not make external model calls.
- Create docs/diffs/DIFF-128-*.md.

DIFF-128 verification:
- git status --short
- git diff --check
- cargo fmt --all --check
- cargo clippy --workspace --all-targets
- cargo test --workspace
- cargo test -p igy6-gateway if gateway changed
- npm --prefix apps/web run build
- npm --prefix apps/web run test:ui-smoke
- npm --prefix apps/web test
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- docker compose -f infra/docker-compose.yml --env-file .env.example config

DIFF-129: UI controls for model/provider/status

Goal:
Add user-facing UI controls and docs for local LLM provider status and model selection/config display.

Required UI behavior:
- Settings must show LLM provider status.
- Assistant must show whether answer mode is:
  - deterministic evidence
  - local LLM evidence-grounded
  - unavailable
- Settings must show:
  - provider: none or ollama
  - Ollama base URL, redacted/safe if needed
  - model name
  - health status
  - timeout
  - evidence-required status
- UI must not require secrets.
- UI must not expose tokens.
- UI must not imply external calls are happening.
- UI must clearly state:
  - local-first
  - no external model by default
  - evidence required
  - deterministic fallback available
- Add examples:
  - Normal user: “Use local model to summarize uploaded warranty note using only evidence.”
  - Coder: “Use local model to explain build log failure with citations.”
- Add Advanced controls only for raw provider diagnostics.
- Update README.md and docs/user-guide.md.
- Update UI smoke test coverage to include LLM status copy.
- Create docs/diffs/DIFF-129-*.md.

DIFF-129 verification:
- git status --short
- git diff --check
- npm --prefix apps/web run build
- npm --prefix apps/web run test:ui-smoke
- npm --prefix apps/web test
- cargo fmt --all --check if Rust changed
- cargo clippy --workspace --all-targets if Rust changed
- cargo test --workspace if Rust changed
- python3 scripts/rust-route-parity.py --check
- scripts/rust-cutover.sh --check
- docker compose -f infra/docker-compose.yml --env-file .env.example config

Final response after DIFF-129:
Report:
- DIFFs completed: 123 through 129, or exact stopping point if blocked.
- Commit hash for each DIFF.
- Push status for each DIFF.
- Runtime smoke script/docs status.
- E2E manual upload smoke status.
- Worker/processing verification status.
- LLM provider plan status.
- Ollama adapter status.
- Evidence-grounded LLM answer status.
- LLM UI controls/status page status.
- Whether local Ollama is required to be running for tests.
- Whether external model calls are still blocked by default.
- Whether deterministic evidence fallback still works.
- Current route parity:
  - web_requires_fallback
  - missing_from_rust
  - FastAPI fallback required?
- Verification results for each DIFF.
- True hard blocker details if stopped early.

Start now. Continue DIFF by DIFF until DIFF-129 is complete or a true hard blocker occurs.
```
