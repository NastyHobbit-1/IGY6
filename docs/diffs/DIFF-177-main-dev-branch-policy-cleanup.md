# DIFF-177: Main Dev Branch Policy Cleanup

Status: Draft

## Type

Documentation / governance cleanup

## Objective

Document the current branch boundary after DIFF-176 was promoted from `dev` to `main` through a clean cherry-picked branch.

The goal is to prevent dev-only build-agent instruction files from being merged into `main` while still allowing runtime/product work from `dev` to be promoted safely.

## Baseline Facts

- `main` is currently at DIFF-176.
- `dev` contains DIFF-176 plus dev-only instruction/build files.
- DIFF-176 was promoted to `main` through `diff-176-clean-main`, not by merging `dev`.
- `dev` must not be merged directly into `main`.

## Allowed Scope

- Add `docs/BRANCH_POLICY.md`.
- Add this DIFF record.
- Optionally update product documentation to link to `docs/BRANCH_POLICY.md`.

## Prohibited Scope

- Do not merge `dev` into `main`.
- Do not add `.codex` to `main`.
- Do not add root `AGENTS.md` to `main`.
- Do not add private build-agent prompts to `main`.
- Do not change runtime code.
- Do not change Docker Compose.
- Do not mutate `.env`.
- Do not touch runtime/private data.
- Do not edit locked DIFFs.

## Verification

- `git status --short`
- `git diff --check`
- `git diff --name-status main..HEAD`
- Forbidden-file check for dev-only instruction files.

## Completion Criteria

- Branch policy is documented.
- Main/dev boundary is explicit.
- Clean promotion process is documented.
- Forbidden main files are listed.
- No runtime code changes are included.

## Result

- Added `docs/BRANCH_POLICY.md`.
- Documented that `main` is the product/runtime branch.
- Documented that `dev` may contain development-only build instructions and must not be merged directly into `main`.
- Documented clean promotion rules for runtime/product work from `dev` to `main`.
- Listed files forbidden on `main` unless a later DIFF explicitly changes policy.

## Verification Result

Pending local verification after branch sync.
