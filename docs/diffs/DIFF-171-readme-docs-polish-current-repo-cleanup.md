# DIFF-171: README Docs Polish Current Repo Cleanup

Status: Locked

## Type

Change-bearing documentation cleanup

## Objective

Polish `README.md` as the current repository entry point and audit active
runtime/migration docs for stale post-cutover runtime claims.

## Baseline Facts

- DIFF-166 locked post-cutover hardening.
- DIFF-168 added `scripts/post-cutover-smoke.sh --check`.
- DIFF-169 added `scripts/fresh-clone-startup-check.sh --check`.
- DIFF-170 added `scripts/runtime-lifecycle-check.sh --check`.
- Rust-only application API/worker runtime remains claimed.
- Python/FastAPI fallback is inactive and archived.
- Python/Celery worker is inactive and archived.
- Celery beat is inactive.
- Runtime/private data remains outside the repo under `IGY6_DATA_ROOT`.

## Allowed Scope

- Update current README/runtime/migration/plan docs.
- Clarify active runtime posture versus archived legacy history.
- Add this DIFF record.
- Do not change runtime ownership, Docker Compose, code, `.env`, archives, or
  runtime/private data.

## Prohibited Scope

- Do not change runtime code.
- Do not change Docker Compose.
- Do not mutate `.env`.
- Do not touch runtime/private data.
- Do not remove archive files.
- Do not edit locked DIFFs.
- Do not start DIFF-172.
- Do not perform UI feature work.
- Do not make broad refactors.
- Do not claim non-Rust infrastructure was rewritten in Rust.
- Do not remove historical DIFF records.

## Required Tags

Use `DIFF-171` in the final change summary and any commit or review note.

## Verification

- `git status --short`
- `git diff --check`
- `python3 -m json.tool configs/rust-cutover-manifest.json`
- `python3 -m json.tool configs/legacy-fastapi-route-classification.json`
- `python3 scripts/post-cutover-runtime-audit.py`
- `scripts/post-cutover-smoke.sh --check`
- `scripts/fresh-clone-startup-check.sh --check`
- `scripts/runtime-lifecycle-check.sh --check`
- `scripts/rust-cutover.sh --check`
- `docker compose -f infra/docker-compose.yml --env-file .env.example config`
- `npm --prefix apps/web run build`
- repo search for stale active Python/FastAPI/Celery runtime claims

## Completion Criteria

- README clearly explains IGY6, current runtime status, quickstart, validation,
  runtime data boundaries, active services, archived legacy code, docs map,
  troubleshooting, development rules, and locked DIFF discipline.
- Active docs no longer contain stale active FastAPI/Python/Celery/beat claims.
- Historical migration/DIFF references remain preserved and clearly
  distinguished from active runtime posture.

## Result

- Reworked `README.md` into the current repository entry point with project
  purpose, Rust-only API/worker runtime posture, quickstart, validation
  commands, runtime data rules, active service overview, archived legacy Python
  explanation, docs map, troubleshooting, rollback posture, and DIFF rules.
- Updated active runtime/migration docs to point to the DIFF-168 through
  DIFF-170 validation ladder and to distinguish historical migration entries
  from current runtime posture.
- Fixed stale current wording in the non-web FastAPI route classification doc:
  Rust-only application API/worker runtime is now claimed after DIFF-165.
- Runtime ownership did not change.
