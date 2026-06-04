# DIFF-205 - Evidence-Aware Task Planner Suggestions

Status: Complete

## Purpose

Make task planning evidence-aware. The planner should tell the user whether enough evidence is present, what evidence appears relevant, and what evidence is missing before work/action proceeds.

## Files Inspected

- `docs/diffs/DIFF-204-approval-gated-plan-to-work-item-flow.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/api/chat/retrieval-preview/route.ts`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-retrieval-preview/src/lib.rs`

## Evidence-Aware Planning Behavior

Added a UI-only evidence check to the existing agent task planner:

- New planner button: `Check evidence`
- New planner summary panel: `data-agent-planner-evidence`
- Existing route used: `/api/chat/retrieval-preview`
- Existing backend route used by proxy: `/chat/retrieval-preview`

The evidence check is enabled only after `/agent/intent` returns a request understanding preview. It does not execute actions, create work items, dispatch work, change data, or persist anything.

## Retrieval, Empty, And Missing Evidence Handling

The planner evidence check summarizes safe retrieval metadata only:

- `answer_status`
- `retrieved_count`
- safe labels derived from evidence/chunk/document/source IDs
- `missing_evidence` boolean

When retrieval returns no hits, the planner shows missing-evidence guidance:

- no relevant local evidence was retrieved
- add/process data or narrow the request before proceeding

When retrieval returns hits, the planner shows:

- retrieved hit count
- up to five safe evidence labels

The planner intentionally does not display raw uploaded text, raw evidence statements, raw chunk text, full retrieval JSON, secrets, `.env` contents, or `IGY6_DATA_ROOT` contents.

## Verification Commands And Results

- `npm --prefix apps/web run build`: passed.
- `git diff --check`: passed.
- `rg "data-agent-check-evidence|data-agent-planner-evidence|Evidence check summary|missing_evidence|retrieved_count" apps/web/src/app/page.tsx`: found expected markers.
- `scripts/operator-smoke-check.sh --check`: failed at Docker socket permission preflight in this Codex environment.
- `scripts/operator-smoke-check.sh --latest-result || true`: passed and summarized the latest recorded Docker-permission failure.

No Rust files changed in DIFF-205, so Rust formatting/tests were not required for this DIFF.

Full smoke run was skipped because Docker socket access is unavailable in this Codex environment:

```text
permission denied connecting to /var/run/docker.sock
```

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/diffs/DIFF-205-evidence-aware-task-planner-suggestions.md`

## Scope Confirmation

- Retrieval engine behavior changed: no.
- New backend route added: no.
- Planner persistence changed: no.
- Work item creation changed: no.
- Action execution changed: no.
- Raw uploaded text output added: no.
- Autonomous self-improvement added: no.
- Runtime/private data dumped from `IGY6_DATA_ROOT`: no.
- `.env` edited: no.
- Main branch work, merge, cherry-pick, push, or promotion: no.
- Private/dev files remain tracked on `dev`.
