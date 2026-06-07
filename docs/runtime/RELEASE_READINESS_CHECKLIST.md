# Release Readiness Checklist

DIFF-244 adds a Codex-safe release-readiness checklist. It does not promote
files, switch branches, merge, cherry-pick, push, restore runtime data, delete
runtime data, or create a full production backup.

Promotion remains deferred until explicit owner instruction.

## Codex-Safe Checks

Run these checks from `dev`:

```bash
git status --short
git diff --check
bash -n scripts/backup-export-mvp.sh
bash -n scripts/restore-dry-run-mvp.sh
bash -n scripts/diagnostics-bundle-mvp.sh
bash -n scripts/normal-user-product-smoke.sh
scripts/restore-dry-run-mvp.sh --bundle tests/fixtures/backup-export-safe-bundle-v1.json --strict-safety
scripts/diagnostics-bundle-mvp.sh --dry-run
scripts/normal-user-product-smoke.sh --release-readiness-check
npm --prefix apps/web run build
```

These checks are non-Docker and do not touch runtime databases or local service
volumes.

## Owner WSL Checks

Run full local-stack verification only in normal WSL:

```bash
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run --record
scripts/operator-smoke-check.sh --latest-result
```

Then follow `docs/runtime/NORMAL_USER_PRODUCT_SMOKE.md` with synthetic data.

## Lifecycle Gates

- Backup export is metadata-only and must pass safety validation before a local
  export file is written.
- Restore is dry-run validation only and must not write to PostgreSQL, artifact
  storage, Qdrant, Neo4j, Redis, MLflow, Phoenix, or runtime data roots.
- Restore dry-run strict safety mode must reject bundles with secret-shaped
  fields, raw content fields, or private path hints.
- Diagnostics bundles must pass self-redaction checks before writing.
- Delete/retention destructive behavior remains unsupported unless a later
  explicit DIFF implements and verifies it.

## Promotion Gate

Do not merge `dev` into `main`.

Do not cherry-pick broad dev commits into `main`.

Do not create promotion branches or promote files until the owner explicitly
requests a later promotion DIFF.
