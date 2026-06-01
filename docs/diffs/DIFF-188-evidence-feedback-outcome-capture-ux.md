# DIFF-188 Evidence Feedback and Outcome Capture UX

Status: Complete

## Branch Policy

- Work happens on `dev`.
- Private/dev/build instruction files stay tracked on `dev`.
- `main` remains the public/runtime-clean branch.
- Public/runtime-safe changes can be promoted to `main` later by explicit
  instruction.
- This DIFF does not merge, cherry-pick, remove private/dev files, touch
  `main`, push, edit `.env`, or start Rust migration work.

## Purpose

Add the smallest honest Results-tab path for recording whether retrieved
evidence or a review target was useful, wrong, incomplete, or otherwise
resolved, using the existing feedback/outcome APIs.

## Files Inspected

- `AGENTS.md`
- `docs/BRANCH_POLICY.md`
- `README.md`
- `docs/ui/README.md`
- `docs/diffs/DIFF-110-rust-feedback-outcome-write-routes.md`
- `docs/diffs/DIFF-068-feedback-outcome-learning-side-effects.md`
- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/rust-migration/POST_CUTOVER_ROUTE_AUDIT.md`

## Current Feedback/Outcome Capability Found

- The Rust gateway exposes:
  - `GET /feedback`
  - `GET /feedback/{feedback_id}`
  - `POST /feedback`
  - `GET /outcomes`
  - `GET /outcomes/{outcome_id}`
  - `POST /outcomes`
- `POST /feedback` persists review metadata and supports targets including
  `evidence_item`, `document`, `source`, `report`, and `work_item`.
- `POST /feedback` validates labels such as `useful`, `not_useful`, `wrong`,
  `verified`, `incomplete`, `noisy`, `trusted`, and `rejected`.
- `POST /outcomes` persists outcome records, validates target existence, and is
  limited to prediction, recommendation, work item, hypothesis, pattern, and
  report targets.
- Outcome statuses include `correct`, `wrong`, `useful`, `not_useful`,
  `partial`, `inconclusive`, `confirmed`, and `disconfirmed`.
- Before this DIFF, feedback and outcome write controls existed in the Advanced
  route console and recent records were visible under Safety & Audit, but normal
  Results review did not expose a direct capture path.

## UX/API Changes Made

- Added a Results-tab `Review outcome capture` section.
- The feedback form offers only currently visible API-supported targets from
  evidence items, reports, and work items.
- The feedback form writes to the existing `POST /feedback` route and supports
  `useful`, `verified`, `incomplete`, `wrong`, and `not_useful` labels.
- The outcome form is shown against only supported outcome targets: reports and
  work items.
- The outcome form writes to the existing `POST /outcomes` route and supports
  `useful`, `correct`, `partial`, `wrong`, `not_useful`, and `inconclusive`
  statuses.
- The UI displays persisted record id, target, and label/status after a
  successful write.
- No fake persistence or dead recovery controls were added.
- Fixed a narrow Rust gateway bug where outcome insertion tried to serialize an
  optional timestamp directly as `timestamptz`; the route now casts nullable
  text to `timestamptz` during insertion.
- No database schema or response contract changes were made.

## Persistence Verification Result

- Feedback persistence was verified with synthetic review text against evidence
  item `evidence-18b4f021284bf95f-3`.
- Persisted feedback id: `feedback-18b4f07cee48dea3`.
- Feedback detail read returned HTTP 200.
- Outcome persistence initially failed with HTTP 502:
  `database error: error serializing parameter 5`.
- The failure was fixed inside DIFF-188 by narrowing the outcome timestamp
  insertion cast.
- Outcome persistence then passed against report `report-18b4f02145b3978f` with
  evidence id `evidence-18b4f021284bf95f-3`.
- Persisted outcome id: `outcome-18b4f0b2d29f03e0`.
- Outcome detail read returned HTTP 200.
- Retrieval regression check returned HTTP 200 with `items=5` and
  `status=retrieved`.
- Browser automation was not available in this run, so verification used the web
  build, live stack probes, API writes/detail reads, and curl/grep page marker
  checks.

## Files Changed

- `apps/web/src/app/page.tsx`
- `crates/igy6-gateway/src/lib.rs`
- `docs/diffs/DIFF-188-evidence-feedback-outcome-capture-ux.md`

## Verification Summary

- `git status --short` before DIFF-188: clean after DIFF-187 commit.
- `docker compose -f infra/docker-compose.yml --env-file .env config --quiet`:
  passed.
- Non-secret `IGY6_DATA_ROOT` key presence check: passed.
- `../IGY6_Data` directory existence check: passed.
- `npm --prefix apps/web run build`: passed.
- `cargo fmt --all --check`: passed.
- `cargo test -p igy6-gateway`: passed, 63 tests.
- Port conflict check before stack start: no listeners found on 3000, 8000, or
  8765.
- `scripts/run.sh`: started the local stack for live checks.
- API ready probe: HTTP 200.
- Web root probe: HTTP 200.
- Results UI marker check: `data-evidence-feedback-workflow` present.
- Feedback API persistence: passed.
- Outcome API persistence: failed before the Rust fix, passed after the fix and
  API rebuild.
- Results/retrieval regression: passed.
- `scripts/stop.sh`: stopped the stack.
- Final port check: no listeners found on 3000, 8000, or 8765.
- Private/dev files remained tracked.
- No merge, cherry-pick, push, `main` work, `.env` edit, Docker Compose edit,
  schema migration, or broad refactor was performed.

## Final Status

DIFF-188 is complete.
