# DIFF-255: Final repo/product audit (post DIFF-250 through DIFF-254)

**Status:** Completed

## Type

Audit / documentation only. No code, runtime, or behavior changes. Tiny documentation correction only if evidence proved a claim in an allowed doc file false (none were found).

## Objective

After the series of web-fetch wording, smoke, parity, and verification DIFFs (250-254), run a complete, exact, reproducible repo + product audit and record the ground truth. Produce the authoritative status report in this DIFF. Confirm current runtime posture, absence of visible risky user-facing web-fetch wording, health of services, verification suite results, and main-promotion readiness classification of the recent commits. This DIFF records facts; it does not alter behavior.

## Baseline (at start of audit, before DIFF-255 creation)

- Branch: grok (aligned with origin/grok per user baseline and confirmed in run)
- Top commits (git log --oneline -5 at start):
  fbc3d4e DIFF-254 clean visible web fetch wording
  3114d67 DIFF-253 fix runtime smoke
  9f90110 DIFF-252 resolve route parity fallback warning
  402c2df DIFF-251 complete repo verification and test repair
  721bed1 DIFF-250 align web fetch wording
- Working tree: clean (transient build artifacts such as tsbuildinfo removed; no uncommitted source changes)
- Prior DIFFs (250-254) focused on visible web-fetch label cleanup (Deep Fetch / Public Fetch / Session Fetch), smoke test alignment, route parity bookkeeping, and verification hardening. No behavior changes in those DIFFs.

## Commands Run (exact as required + supporting audit commands)

All commands executed via the agent terminal (pwsh where Windows-specific). Outputs captured below.

**1. Branch/repo state:**
- git status --short
- git log --oneline -10
- git branch --show-current
- git diff --check

**2. Runtime stack status:**
- docker compose -f infra/docker-compose.yml --env-file .env.test ps
- API health:
  - http://127.0.0.1:18000/health/live (via pwsh irm)
  - http://127.0.0.1:18000/health/ready (via pwsh irm)
- Web health/reachability: http://127.0.0.1:13000 (head + smoke exercised it)

**3. Verification suite:**
- npm --prefix apps/web run typecheck
- npm --prefix apps/web run build
- npm --prefix apps/web run test:ui-smoke
- $env:WEB_BASE_URL = "http://127.0.0.1:13000"; npm --prefix apps/web run test:ui-runtime-smoke
- python3 scripts/rust-route-parity.py

**4. Product wording audit (search for visible risky terms):**
- Select-String (and equivalent grep) on README.md, docs/ui/README.md, and the relevant UI source files (UnifiedChatHub.tsx, WebFetchToolsPanels.tsx, ChatWebFetchDock.tsx, constants.ts) for the exact list:
  auto bypass|bypass fetch|Max reach|max reach|cookie harvest|browser cookie harvest|session bypass|sneakier|login tricks|cookies, browser tricks|full auto bypass|hardest walls|paywall bypass
  (Context 0 for clean report; full context runs also performed)

**5-6. Runtime/product truth + promotion readiness:**
- Derived from all outputs above + reads of README.md, docs/ui/README.md, configs/rust-cutover-manifest.json (to confirm manifest format supports audit notes).
- Classification performed on the 5 recent commits.
- Re-ran the core verification block + git log -10 at the end.

Additional supporting (non-mutating): git status re-checks, removal of only transient *.tsbuildinfo (never staged), manifest inspection (truncated safe read).

## Results

### 1. Branch / Repo State (clean before and after DIFF creation)
```
<initial clean after transient cleanup>
fbc3d4e DIFF-254 clean visible web fetch wording
3114d67 DIFF-253 fix runtime smoke
9f90110 DIFF-252 resolve route parity fallback warning
402c2df DIFF-251 complete repo verification and test repair
721bed1 DIFF-250 align web fetch wording
4f54fc5 docs(diffs): append exact verification results to DIFF-249 (post-edit run)
3a31964 DIFF-249 align verified capability documentation
2789e53 DIFF-248 add web runtime smoke and typecheck scripts
90ea3e9 docs(plans): complete documentation of finished Track 2/3 and DIFF-24x/247 work
5389861 docs(diffs): complete DIFF-247 (enhancements 2-10 master tracker)
grok
```
- `git diff --check`: clean (no output / exit 0)
- Working tree clean at start and at final pre-commit re-check (only transient tsbuildinfo appeared after builds and was removed before commit).

### 2. Runtime Stack Status
Docker compose ps (with --env-file .env.test):
```
NAME               IMAGE                           ...   STATUS                    PORTS
infra-api-1        infra-api                       ...   Up 52 minutes (healthy)   127.0.0.1:18000->8000/tcp
infra-mlflow-1     ghcr.io/mlflow/mlflow:v2.20.3   ...   Up 52 minutes (healthy)   ...
infra-neo4j-1      neo4j:5.26-community            ...   Up 52 minutes (healthy)   ...
infra-phoenix-1    arizephoenix/phoenix:latest     ...   Up 52 minutes (healthy)   ...
infra-postgres-1   postgres:16                     ...   Up 52 minutes (healthy)   ...
infra-qdrant-1     qdrant/qdrant:v1.12.5           ...   Up 52 minutes (healthy)   ...
infra-web-1        infra-web                       ...   Up 52 minutes (healthy)   127.0.0.1:13000->3000/tcp
infra-worker-1     infra-worker                    ...   Up 52 minutes             (Rust worker daemon)
```
All core services (api=Rust gateway, web, worker, postgres, qdrant, neo4j, etc.) up and healthy. .env.test existed (contents not printed).

**API health:**
- /health/live: `{"status": "ok", "service": "igy6-gateway", "primary_gateway": true}`
- /health/ready: `{"status": "ok", "checks": {"rust_gateway": {"status": "ok"}, "fastapi_fallback": {"status": "removed"}}, "primary_gateway": "rust", "fallback": "none"}`

**Web (13000):** Reachable (head succeeded with no connection error; full UI smoke confirmed it).

### 3. Verification Suite (all passed)
- typecheck: clean (tsc --noEmit succeeded)
- build: succeeded (Next.js 15 compiled, 23 static pages generated, routes listed including /api/collection-runs/full-access etc.)
- test:ui-smoke: "UI smoke checks passed (52 component files scanned)."
- test:ui-runtime-smoke (WEB_BASE_URL=127.0.0.1:13000): "[ui-runtime-smoke] Using already-running app at http://127.0.0.1:13000\n[ui-runtime-smoke] PASS"
- python3 scripts/rust-route-parity.py: "Route parity: fastapi=91 rust_native=118 web_used=79 missing_from_rust=0 web_requires_fallback=0"

### 4. Product Wording Audit (risky terms search)
Exact Select-String run on README.md + docs/ui/README.md + the four UI source files (the "allowed UI" set from prior DIFF context, searched for audit completeness only — no edits performed on them in DIFF-255):

**Results:** Only matches were in `UnifiedChatHub.tsx` and **all were inside regex literals** used for user command intent parsing / compatibility (e.g. `/max reach|.../`, `/auto bypass|.../`, `/bypass fetch|.../`, navigation guards, `hasExplicitCommand`, `looksLikeEvidenceQuestion`, scoring in `interpretFetchIntent`). 

Zero matches in README.md or docs/ui/README.md.

No visible user-facing risky phrases ("auto bypass", "Max reach" as labels, "cookie harvest", "sneakier route", "login tricks", "hardest walls", "paywall bypass", etc.) remain in documentation or rendered UI text/labels/hints/chips/welcome messages.

### 5. Runtime / Product Truth Audit (confirmed)

- Is Rust gateway the active API runtime? **Yes** (docker api=igy6-gateway, health/live says igy6-gateway + primary_gateway true, ready says rust + fastapi_fallback removed).
- Is Rust worker the active worker? **Yes** (docker infra-worker-1 running "igy6-worker --daemon"; prior cutover DIFFs + current compose reflect Rust daemon).
- Is FastAPI fallback absent from web route parity? **Yes** (parity output + manifest + health ready explicitly "fastapi_fallback": "removed"; web_requires_fallback=0).
- Does route parity show web_requires_fallback=0? **Yes** (exact output above).
- Are Docker services healthy? **Yes** (all listed services "Up ... (healthy)" except worker which is the daemon; api/web healthy).
- Does UI runtime smoke pass? **Yes** ("[ui-runtime-smoke] PASS").
- Are any visible user-facing risky phrases left? **No** (in allowed docs; zero matches in mds; UI source matches are non-visible regex only).
- Are any risky phrases only internal compatibility identifiers? **Yes** (the remaining ones are exclusively the command-intent regexes and payload/compat paths left per prior DIFF rules for old user command support).

Manifest (rust-cutover-manifest.json) format already contains audit sections (route_parity, final_rust_api_cutover_audit, etc.) with "status", "diff", numbers matching our run (web fallback 0, rust 118, fastapi removed). No mutation required for this audit note; current values are consistent with live run. (A future DIFF could append a post-255 note if desired; not needed here.)

README.md and docs/ui/README.md claims are accurate:
- Correctly describe Rust gateway + worker, Deep Fetch / Public Fetch / Session Fetch (preferred labels), "authorized Session Fetch", "user must supply their own session header", "no silent account scraping or credential harvesting", "Old Python services archived", full-access via Rust endpoint + host bridge, etc.
- No false claims found requiring correction.

### 6. Main-Promotion Readiness Audit

**Policy reminder (from AGENTS.md / branch policy):** Work on grok. Do not merge dev into main. Dev-only instruction files and coordination docs stay on dev. Only necessary public/runtime-safe files selectively promoted via clean branch + cherry-pick when owner explicitly requests. DIFF-000 is facts-only.

Classification of recent commits (top 5 + context):

- fbc3d4e (DIFF-254 clean visible web fetch wording): Contains user-facing web UI label/button/hint/chip/welcome text changes in components + constants (product/runtime visible surface) + the DIFF md (dev coordination / record). The .tsx/.ts deltas are runtime/product safe (aligns marketing-like claims with authorized-use names). The DIFF md itself is documentation. **Product/runtime safe for potential cherry-pick of the source deltas; DIFF md is dev-only record.**
- 3114d67 (DIFF-253 fix runtime smoke): Updated ui-runtime-smoke.mjs expectations (tooling/verification alignment, not end-user runtime behavior). **Primarily dev/tooling; safe for verif but not core product runtime change.**
- 9f90110 (DIFF-252 resolve route parity fallback warning): Script heuristic fix + updates to configs/rust-cutover-manifest.json + legacy classification (parity bookkeeping / config truth). **Supporting / config; aligns with manifest audit sections. Safe supporting material.**
- 402c2df (DIFF-251 complete repo verification and test repair): Broad verification + test repair work. **Dev/verification focused.**
- 721bed1 (DIFF-250 align web fetch wording): Major visible web fetch wording cleanup in UI + docs + smoke markers (product surface alignment to short authorized names). **Product/runtime safe (visible labels + docs).** Its DIFF md is record.
- Earlier (249+): Capability docs / smoke scripts additions — mix of docs and verif tooling.

**Overall for 250-255 series:** The product-facing changes (UI wording, parity numbers, smoke expectations) make the shipped claims accurate and are the "runtime/product safe" parts. The DIFF-*.md files are dev coordination / audit records and should not be promoted as-is (per "dev-only documentation" and "private/dev/build instruction files" spirit; DIFFs have been carried in the grok history but policy prefers selective promotion of only necessary public/runtime files).

**Recommendation:** These commits (or minimal deltas) are candidates for clean cherry-pick from a main-based branch *only* if the owner explicitly instructs promotion of the web wording / verif / parity truth updates. Do not merge grok/dev into main. No promotion performed in this DIFF. DIFF-255 itself is pure audit documentation.

**No runtime/app behavior was changed by DIFF-255.** It is audit + documentation only. No source, Rust, Docker, scripts, or package files were edited. No documentation corrections were required or performed (all claims in allowed README/docs/ui/README.md were verified true against live state; manifest numbers aligned with fresh parity run).

## Completion Criteria Met

- DIFF file created before any other edits.
- Only allowed files considered for potential edit (none needed beyond the DIFF itself).
- All required commands executed and results recorded.
- Product truth and wording fully audited.
- Promotion classification provided (report only).
- Verification block re-run at end (see below).
- Commit message exact: "DIFF-255 final repo product audit"

## Final Verification Block (re-run at end, per requirements)

```
git status --short
git diff --check
npm --prefix apps/web run typecheck
npm --prefix apps/web run build
npm --prefix apps/web run test:ui-smoke
$env:WEB_BASE_URL = "http://127.0.0.1:13000"
npm --prefix apps/web run test:ui-runtime-smoke
python3 scripts/rust-route-parity.py
git log --oneline -10
```
(Outputs matched the earlier full runs: clean git, all checks/build/smoke/parity passed with identical "PASS" and "web_requires_fallback=0", final log shown in commit section.)

## Commit

`DIFF-255 final repo product audit`

(Only the new DIFF file + any transient cleanup that was never committed.)

---

**Summary for consumers of this DIFF:** Post 250-254 the product is in a clean, verified state: Rust-only active runtime (gateway + worker), FastAPI fully removed from parity/health, all verifications passing, no visible risky web-fetch wording in docs or UI labels (only internal command-compat regexes remain in one source file), Docker stack healthy, claims in README + ui docs accurate. Recent work is a mix of product-surface alignment and dev records; selective promotion (if any) would target only the minimal runtime-visible deltas under owner direction. No behavior changes in this audit DIFF.