# Operator Smoke Verification Bundle

This bundle is the repeatable local operator check for the verified
manual-upload evidence path:

```text
compose config -> data-root presence -> web build -> stack start -> live/ready
-> manual upload smoke -> work/evidence/retrieval/results checks -> stack stop
```

It is local-first and uses synthetic text only. It must not print `.env`
contents, secrets, runtime artifacts, or files under `IGY6_DATA_ROOT`.

## Automated Script

Use the operator smoke script from the repository root:

```bash
scripts/operator-smoke-check.sh --help
scripts/operator-smoke-check.sh --check
scripts/operator-smoke-check.sh --run
```

`--check` verifies prerequisites and configuration only. It checks required
commands, required repo files, Compose config, `IGY6_DATA_ROOT` key presence
without printing the value, `IGY6_DATA_ROOT` directory presence without listing
contents, and the checked host ports. It does not start the stack and does not
mutate runtime data.

`--run` performs the full local smoke path with synthetic text. It runs the web
build, checks ports before startup, starts the stack, probes API live/ready and
the web UI, calls `scripts/e2e-manual-upload-smoke.py --run`, checks the
Results UI markers, verifies retrieval preview, stops the stack if the script
started it, and confirms the checked ports are clear.

Success means every step prints `PASS` and the script exits `0`. Failure means
one or more steps print `FAIL` and the script exits nonzero. The script does not
print `.env` contents, secret values, runtime artifact contents, or private data
from `IGY6_DATA_ROOT`; it uses only synthetic smoke data.

## Preconditions

- Run from the repository root on `dev`.
- Use a local stack only.
- Do not run against private user data unless the owner explicitly authorizes
  that runtime.
- Browser automation is optional. If unavailable, use the curl/grep marker
  checks below.

## Commands

1. Check the working tree and branch:

```bash
git status --short
git branch --show-current
git branch -vv
```

2. Validate Compose without printing secrets:

```bash
docker compose -f infra/docker-compose.yml --env-file .env config --quiet
grep -q '^IGY6_DATA_ROOT=' .env && echo "IGY6_DATA_ROOT key present" || echo "IGY6_DATA_ROOT key missing"
test -d ../IGY6_Data && echo "IGY6_DATA_ROOT directory exists" || echo "IGY6_DATA_ROOT directory missing"
```

3. Build the web UI:

```bash
npm --prefix apps/web run build
```

4. Check for port conflicts:

```bash
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
```

5. Start the stack:

```bash
scripts/run.sh
```

6. Probe the API and web UI:

```bash
curl -s -o /tmp/igy6-operator-live.json -w '%{http_code}\n' http://127.0.0.1:8000/health/live
curl -s -o /tmp/igy6-operator-ready.json -w '%{http_code}\n' http://127.0.0.1:8000/health/ready
curl -s -o /tmp/igy6-operator-page.html -w '%{http_code}\n' http://127.0.0.1:3000/
```

Success means all three commands print `200`.

7. Run the synthetic manual-upload smoke:

```bash
python3 scripts/e2e-manual-upload-smoke.py --run
```

Success means the script prints PASS lines for live/ready/web, source,
permission, approval, manual upload, raw artifact, normalization work item,
work item status, evidence endpoints, and retrieval preview. The script may
warn when the worker has not completed processing yet; rerun after processing
if needed.

8. Verify normal-user UI markers with curl/grep:

```bash
grep -q 'data-guided-manual-result' /tmp/igy6-operator-page.html && echo "guided upload marker present" || echo "guided upload marker missing"
grep -q 'data-work-status-item' /tmp/igy6-operator-page.html && echo "work status marker present" || echo "work status marker missing"
grep -q 'data-chat-preview-results' /tmp/igy6-operator-page.html && echo "retrieval UI marker present" || echo "retrieval UI marker missing"
grep -q 'data-basic-report-workflow' /tmp/igy6-operator-page.html && echo "report workflow marker present" || echo "report workflow marker missing"
grep -q 'data-evidence-feedback-workflow' /tmp/igy6-operator-page.html && echo "feedback workflow marker present" || echo "feedback workflow marker missing"
grep -q 'data-source-evidence-history' /tmp/igy6-operator-page.html && echo "source history marker present" || echo "source history marker missing"
```

If the page was fetched before the smoke run created records, refetch it first:

```bash
curl -s -o /tmp/igy6-operator-page.html -w '%{http_code}\n' http://127.0.0.1:3000/
```

9. Verify retrieval explicitly:

```bash
curl -s -X POST http://127.0.0.1:8000/chat/retrieval-preview \
  -H 'Content-Type: application/json' \
  --data '{"message":"Find blue-raven-117 in my uploaded evidence.","limit":5}' \
  -o /tmp/igy6-operator-retrieval.json \
  -w '%{http_code}\n'
```

Success means HTTP `200`; inspect only counts or status, not private runtime
content:

```bash
python3 -c "import json; p=json.load(open('/tmp/igy6-operator-retrieval.json')); print('items=' + str(len(p.get('items', []))) + ' status=' + str(p.get('answer_status') or p.get('status') or 'ok'))"
```

10. Stop the stack:

```bash
scripts/stop.sh
```

11. Confirm the checked ports are clear:

```bash
ss -ltnp 2>/dev/null | grep -E ':3000|:8000|:8765' || true
```

No output means the checked ports are clear.

## Failure Handling

- If Compose config fails, stop and fix configuration before starting the stack.
- If `IGY6_DATA_ROOT` is missing, do not create ad hoc runtime folders inside
  the repository.
- If a port is already listening, stop the conflicting local process or choose a
  documented alternate only if the repo supports it.
- If manual upload succeeds but retrieval has no hits, check Work for pending or
  failed processing before treating retrieval as broken.
- If a marker is missing, rebuild the web UI and refetch `/` before debugging
  runtime APIs.
- Always stop the stack after verification unless the owner asks to keep it
  running.
