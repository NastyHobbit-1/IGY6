# DIFF-071: Web TypeScript Config Normalization

Status: Locked

## Type

Change-bearing.

## Objective

Persist the TypeScript configuration expected by Next.js so web builds do not
generate untracked configuration drift.

## Baseline Facts

- DIFF-000 through DIFF-070 are locked.
- `npm --prefix apps/web run build` succeeds but generates `apps/web/tsconfig.json`
  when it is absent.
- Next.js also updates `apps/web/next-env.d.ts` to include generated route
  types.

## Allowed Scope

This DIFF allows changes only to:

- `docs/diffs/DIFF-071-web-typescript-config-normalization.md`
- `apps/web/tsconfig.json`
- `apps/web/next-env.d.ts`

Allowed behavior:

- Add the Next.js-generated TypeScript config.
- Update `next-env.d.ts` to the Next.js expected content.

## Prohibited Scope

This DIFF does not allow application behavior changes, UI changes, dependency
changes, backend changes, Docker changes, or broad refactors.

## Required Tags

Use `DIFF-071` in change summaries, commits, and review notes.

## Verification

Required checks:

```bash
npm --prefix apps/web run build
git diff --check
```

Completed verification:

```bash
npm --prefix apps/web run build
git diff --check
```

## Completion Criteria

This DIFF is complete when web TypeScript configuration is explicit and
verification passes.
