# DIFF-121: Web Workflow UI Reorganization

Status: Locked

## Goal

Reorganize the IGY6 web UI around user workflows instead of backend object
categories, and update user-facing documentation with practical examples for
normal PC users and seasoned coders.

## Scope

- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/layout.tsx` only for page metadata if useful
- `README.md`
- User-facing docs that describe the old web navigation or old split between
  Chat, Agent Command, Sources, Evidence, Memory, Approvals, and Audit

## Required UI Sections

- Home
- Assistant
- Data & Knowledge
- Work & Processing
- Reports
- Safety & Audit
- Settings

## Required Behavior

- Combine Chat and Agent Command into Assistant.
- Combine Sources, Uploads, Evidence, Memory, and Analysis into Data &
  Knowledge.
- Combine Approvals, Audit, and safety policy/status into Safety & Audit.
- Keep Work & Processing, Reports, Settings, and Home separate.
- Keep existing API route calls where possible.
- Preserve local-first, evidence-only, no-external-model, and approval-required
  safety language.
- Keep advanced/debug controls available behind Advanced sections.
- Hide raw IDs, raw JSON, route/debug details, and approval IDs from primary
  user flows unless Advanced is expanded.
- Add plain-English helper text, examples, placeholders, disabled states, and
  next-step guidance for primary forms.

## Prohibited

- No backend behavior changes unless required for UI compatibility.
- No backend route removal.
- No FastAPI removal.
- No Rust-only claim unless manifest and route parity prove it.
- No broad refactors outside the files above.
- No locked DIFF edits.
- No secrets or runtime/private data commits.
- No new action execution surface or broader agent behavior.
- No unsafe deletion.

## Documentation

Update README and relevant user-facing docs with:

- Current Rust-primary backend posture and honest FastAPI fallback status.
- Long Docker Compose start/stop/status commands.
- Simple WSL aliases: `igy6-start`, `igy6-stop`, `igy6-ps`, `igy6-logs`.
- Warning that `down -v` is not a normal stop command and can delete Docker
  volume data.
- New UI navigation overview.
- Normal PC user quickstart and examples.
- Seasoned coder quickstart and examples.
- Manual upload test flow.
- Safety and approvals explanation.
- Troubleshooting and verification commands.

## Verification

- `git status --short`
- `git diff --check`
- `npm --prefix apps/web run build`
- Run existing web tests if present
- `python3 scripts/rust-route-parity.py --check` if route usage changes
- `python3 scripts/test-rust-route-parity.py` if route parity logic changes
- `cargo fmt --all --check` if backend/Rust files change
- `cargo clippy --workspace --all-targets` if backend/Rust files change
- `cargo test --workspace` if backend/Rust files change
- `scripts/rust-cutover.sh --check` if manifest/runtime docs change
- `python3 -m json.tool configs/rust-cutover-manifest.json` if changed
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json` if
  changed
- Validate changed snippet-vault JSONL files line-by-line as valid JSON if
  snippet-vault changes
- `docker compose -f infra/docker-compose.yml --env-file .env.example config` if
  Compose/runtime wiring changes

## Completion Notes

- Reorganized the web UI into Home, Assistant, Data & Knowledge, Work &
  Processing, Reports, Safety & Audit, and Settings.
- Combined Chat Retrieval Preview and Agent Command into Assistant.
- Combined source, upload, collection, evidence, memory, analysis, and search
  records into Data & Knowledge.
- Moved raw IDs, raw JSON, approval IDs, route/debug detail, and legacy route
  forms behind Advanced sections.
- Preserved existing API calls and did not change backend behavior.
- Updated README and user guide with normal PC user and seasoned coder examples,
  local start/stop commands, WSL aliases, safety notes, and honest Rust-primary
  posture.

## Verification Results

- Passed: `git diff --check`
- Passed: `npm --prefix apps/web run build`
- Passed: `python3 scripts/rust-route-parity.py --check`
  - `fastapi=91`
  - `rust_native=64`
  - `web_used=45`
  - `missing_from_rust=30`
  - `web_requires_fallback=0`
- Passed: `scripts/rust-cutover.sh --check`
- Not run: additional web tests; `apps/web/package.json` defines no test script.
- Not run separately: Cargo fmt/clippy/test commands; no Rust files changed, and
  `scripts/rust-cutover.sh --check` ran Rust workspace tests and clippy.
- Not run: JSON config validation; no JSON config files changed.
- Not run: Docker Compose config validation; Compose/runtime wiring did not
  change.
