# DIFF-252: Resolve route parity fallback warning

**Status:** Completed

## Type
Verification and minimal fix only.

## Objective
Investigate the persistent `web_requires_fallback=1` reported by `scripts/rust-route-parity.py`, identify the root cause, and apply the smallest correction so that the guard reports 0 (matching the manifest expectation and reality).

## Baseline Facts
- The parity script computes web_used_routes from static analysis of apps/web source (regex for fetch/getJson/postJson) and compares to Rust gateway routes.
- It reported exactly one: "POST /user/status"
- All actual calls to /api/user/status in web code use GET (default fetch or explicit GET in the proxy route.ts).
- Rust gateway only registers GET /user/status.
- The false positive came from a heuristic in web_used_routes: for local_fetch matches, it scans source[match.end(): +240] for 'method: "POST"' to decide GET vs POST. This window can reach *subsequent unrelated* POST fetch calls in the same file (e.g. UserSecurityPanel.tsx has a status GET fetch followed by change-password POST fetch).
- The /user/status GET fetch in the totp check block was being mis-attributed POST because of the later method string in source.
- Updating the manifest/classification counts was also needed for --check to pass (they had drifted from prior feature additions: web_used was 65, rust_native 114; current reality higher due to added web features over time).
- No actual missing Rust route or new capability; purely detection + bookkeeping.

## Allowed Scope
- Edit to `scripts/rust-route-parity.py` (the heuristic window).
- Edits to `configs/rust-cutover-manifest.json` and `configs/legacy-fastapi-route-classification.json` (update the recorded parity numbers to current truth).
- The new DIFF file.
- No changes to any web components, routes, payloads, data-attrs, Rust handlers, etc.

## Prohibited Scope
- No behavior changes.
- No adding the POST /user/status (it is not used).
- No broad refactors to the parity script.
- No locked DIFFs.

## Root Cause
Classification bug in the parity script's crude forward-scan heuristic for inferring HTTP method on plain `fetch("/api/...")` calls. The 240-char window after a GET status check was long enough to include a subsequent `method: "POST"` from a different fetch in the same source file, causing a spurious "POST /user/status" entry in web_requires_fallback.

The 1 was not a real web-used route that lacked Rust support.

## Fix Applied
- Reduced the method-detection lookahead from 240 to 100 characters in the two relevant places (fetch_route and local_fetch loops). 100 chars is sufficient to see an inline options object for the current fetch call but avoids unrelated later statements in the same file.
- Updated the two parity count records in the config/manifest files to the values now correctly emitted by the script (web_used_routes=79, rust_native_routes=118, fallback=0, missing=0). These had become stale as the web app grew.

This makes `python3 scripts/rust-route-parity.py` report `web_requires_fallback=0` and `--check` pass cleanly.

## Verification Commands
See final response for exact outputs of the required list.

All passed with web_requires_fallback=0.

No product capability added or changed. The /user/status route remains GET-only as actually used.

Commit: DIFF-252 resolve route parity fallback warning

## Files Changed
- scripts/rust-route-parity.py (heuristic fix)
- configs/rust-cutover-manifest.json (sync counts)
- configs/legacy-fastapi-route-classification.json (sync counts)
- docs/diffs/DIFF-252-....md (this file)

## Exact Fallback Route Found
"POST /user/status" (spurious; actual usage is GET /user/status via multiple places in web, all proxied as GET, and implemented as GET in Rust gateway).

## Confirmation
The warning is resolved. The parity guard now correctly sees 0 web routes requiring fallback. All verifications (including the parity script itself) pass. Minimal targeted changes only.