# DIFF-075: IGY6 AI Console Visual Shell

Status: Locked

## Type

Change-bearing

## Objective

Restyle the existing IGY6 web UI into a dark ChatGPT/Grok/Open WebUI-style
console shell while preserving all existing IGY6 data loading and behavior.

## Baseline Facts

- The worktree was clean before this DIFF was created.
- No active or in-progress DIFF existed before this DIFF was created.
- The current web page fetches health, sources, collection runs, artifacts,
  documents, chunks, evidence, claims, vector memory, graph schema, patterns,
  hypotheses, predictions, recommendations, work items, approvals, feedback,
  outcomes, reports, and audit events.
- The current UI is a light inventory/dashboard layout.
- `ChatRetrievalPreview` already posts to same-origin `/api/chat/retrieval-preview`
  and must remain retrieval-only with no LLM answer generation or action
  execution.
- `MvpActionConsole` already exposes existing FastAPI-backed controls and must
  keep the same backend calls and behavior.
- The web package has no `lint` script in `apps/web/package.json`; `build` is
  the available web verification script.
- A follow-up scope correction clarified that DIFF-075 must match only the
  visual style/layout language of the user's local AI-stack concept, not any
  ComfyUI, model, image, workflow, download, or AI-stack functionality.

## Allowed Scope

- `docs/diffs/DIFF-075-igy6-ai-console-visual-shell.md`
- `apps/web/src/app/page.tsx`
- `apps/web/src/app/globals.css`
- `apps/web/src/app/layout.tsx` only if needed for visual/layout polish
- Small new UI-only component files under `apps/web/src/app/components/` only if
  necessary to keep `page.tsx` manageable

Allowed behavior area:

- Visual layout, grouping, labels, and CSS presentation of existing loaded data
  and existing UI controls.
- Disabled or clearly scaffolded visual-only controls when needed for shell
  resemblance, without implying functional behavior.

## Prohibited Scope

- No ComfyUI.
- No local AI-stack functionality.
- No model manager.
- No image generation UI.
- No model download/install controls.
- No backend API behavior changes.
- No database changes.
- No migrations.
- No Docker changes.
- No dependency changes.
- No auth system.
- No browser automation.
- No Qdrant, Neo4j, MLflow, or Phoenix changes.
- No source collection changes.
- No approval/work-item behavior changes.
- No self-improvement behavior changes.
- No LLM/model generation.
- No broad refactor.
- No unrelated cleanup.
- No backend file renames.

## Required Tags

Use `DIFF-075` in change summaries, commits, pull requests, and review notes for
this work.

## Verification

Run:

```bash
git diff --check
npm --prefix apps/web run lint
```

If lint is unavailable or not configured, run:

```bash
npm --prefix apps/web run build
```

Also run:

```bash
python3 -m compileall services/api services/worker
```

Do not start Docker unless required by a failing verification step, and record
the reason if that happens.

## Completion Criteria

- The IGY6 web UI visually resembles a modern dark AI-console workspace.
- The UI includes a left navigation/sidebar, chat-first center panel, honest
  status top bar, right context panel, retrieval preview, and preserved MVP
  Action Console.
- Existing data loading remains visible for all previously loaded record types.
- Existing retrieval preview remains functional and retrieval-only.
- Existing MVP Action Console behavior is preserved.
- No ComfyUI or AI-stack functionality is added.
- No backend behavior is changed.
- No dependencies are added.
- Prohibited scope is avoided.
- Verification results are recorded below before locking this DIFF.

## Verification Result

- Passed: `git diff --check`.
- Blocked as expected: `npm --prefix apps/web run lint` because
  `apps/web/package.json` has no `lint` script.
- Passed fallback: `npm --prefix apps/web run build`.
- Passed: `python3 -m compileall services/api services/worker`.
- Not run: Docker/full-stack verification, because this DIFF is visual/UI-only
  and no verification failure required starting services.
- Passed after scope correction: `git diff --check`.
- Blocked as expected after scope correction: `npm --prefix apps/web run lint`
  because `apps/web/package.json` has no `lint` script.
- Passed fallback after scope correction: `npm --prefix apps/web run build`.
- Passed after scope correction: `python3 -m compileall services/api
  services/worker`.

## Out Of Scope Follow-Up

- Real navigation routing.
- New backend endpoints.
- New action behavior.
- Model management or local AI-stack integrations.
- Additional UI improvements after this visual shell.
