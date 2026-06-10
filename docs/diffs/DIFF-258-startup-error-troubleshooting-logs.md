# DIFF-258: Startup and Error Troubleshooting Logs

Status: Complete

## Type

Change-bearing

## Objective

Add persistent startup and error logs under `IGY6_DATA_ROOT/ops/` and expose them in the System UI for easier troubleshooting.

## Allowed Scope

- `docs/diffs/DIFF-258-startup-error-troubleshooting-logs.md`
- `docs/ui/README.md` (troubleshooting logs section only)
- `crates/igy6-cli/src/lib.rs`
- `crates/igy6-gateway/src/lib.rs`
- `crates/igy6-gateway/src/main.rs`
- `apps/web/src/app/api/ops/**`
- `apps/web/src/app/components/product/TroubleshootingLogsPanel.tsx`
- `apps/web/src/app/components/ProductApp.tsx`
- `apps/web/src/app/components/types.ts` (log response types only)
- `apps/web/scripts/ui-smoke.mjs`

## Prohibited Scope

- `.env` / runtime data commits
- Secret exposure in logs
- Framework migration
- Locked DIFF edits

## Verification

- `cargo test -p igy6-cli`
- `cargo test -p igy6-gateway`
- `npm --prefix apps/web run typecheck`
- `npm --prefix apps/web run build`
- `npm --prefix apps/web run test:ui-smoke`
- `git diff --check`

## Completion Criteria

- `igy6 start` writes timestamped startup steps to `ops/startup.log`
- Errors write to `ops/error.log` with redaction
- Gateway exposes bounded `GET /ops/runtime-logs`
- Web System section shows startup/error logs with refresh
- Verification passing

## Result

- `igy6 start` / `igy6 stop` append timestamped lines to `{IGY6_DATA_ROOT}/ops/startup.log` and failures to `ops/error.log` (redacted).
- Rust gateway logs startup and request failures to the same ops files via `append_runtime_ops_log`.
- `GET /ops/runtime-logs` and `POST /ops/runtime-logs/append` expose bounded log reads and UI error recording.
- System tab includes **Startup and error logs** with refresh.

Verification: `cargo test -p igy6-cli`, `cargo test -p igy6-gateway`, `npm --prefix apps/web run typecheck`, `build`, `test:ui-smoke` — all passed.