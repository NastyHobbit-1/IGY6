# DIFF-243 - Guardrails / Tool-Use / External-Model Policy Hardening

Status: Complete

## Branch And Baseline

- Active branch before work: `dev`
- HEAD before work: `b10403f Complete DIFF-242 self improvement experiment workflow MVP`
- `dev` ahead/behind `origin/dev` before work: ahead by three local DIFF commits, not behind
- Controlling plan: `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`

## Scope

This DIFF hardens the local agent/action guardrails and policy visibility. It
does not add dangerous tools, execute shell commands, enable hosted AI, edit
`.env`, or call external services.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `README.md`
- `docs/ui/README.md`
- `crates/igy6-agent-api/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-llm/src/lib.rs`
- `crates/igy6-policy/src/lib.rs`
- `apps/web/src/app/page.tsx`

## Changes

- Hardened the Rust agent classifier to reject:
  - prompt-injection and instruction-override language;
  - hosted/external model requests;
  - provider-name hosted AI requests such as OpenAI, ChatGPT, Claude, and
    Gemini;
  - raw command surfaces and secret-dump/exfiltration wording.
- Unsafe request-understanding now blocks action matching before a known action
  can be selected.
- Added tests proving prompt injection, hosted model requests, and secret dumps
  remain non-executable.
- Extended `/agent/capabilities` with a policy posture object showing:
  - local-first posture;
  - hosted AI disabled;
  - external model policy blocked by default;
  - arbitrary command execution disabled;
  - prompt-injection filter enabled;
  - system-changing approval requirement;
  - blocked request classes.
- Updated the Safety UI to display hosted AI, prompt-injection, tool-use, and
  blocked request-class posture from capabilities.
- Updated `docs/ui/README.md`.

## Verification

- `git status --short` - showed only DIFF-243 scoped changes before commit.
- `git diff --check` - passed.
- `git diff --name-status` - showed:
  - `M apps/web/src/app/page.tsx`
  - `M crates/igy6-agent-api/src/lib.rs`
  - `M crates/igy6-gateway/src/lib.rs`
  - `M docs/ui/README.md`
- `npm --prefix apps/web run build` - passed.
- `cargo fmt --all --check` - passed after applying `cargo fmt --all`.
- `cargo test --workspace` - passed.
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort` - private/dev instruction files remained tracked.
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true` - still reports older out-of-scope draft/template/status-command strings; no new active/in-progress DIFF was left by this DIFF.

## Files Changed

- `crates/igy6-agent-api/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-243-guardrails-tool-use-external-model-policy-hardening.md`

## Notes

- No full Docker smoke was run from Codex per environment rule.
- No runtime/private data was dumped.
- No hosted AI call, hidden external transfer, browser/account scraping,
  credential/cookie/token collection, arbitrary command execution behavior,
  destructive delete, destructive restore, unsafe backup archive, `.env` edit,
  main work, merge, cherry-pick, push, promotion, fake control, or private/dev
  file removal was performed.
