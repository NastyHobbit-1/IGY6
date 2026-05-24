# DIFF-159 Rust Worker Process Ownership Canary Scheduler Beat Posture

## Scope

DIFF-159 adds a safe Rust worker process-ownership canary planning path after
DIFF-158 decided production worker cutover was not ready.

This DIFF does not remove `services/worker/`, does not remove Docker Compose
`worker` or `beat`, does not disable Python/Celery, does not process broad
queues, does not mutate `.env`, does not touch production/private runtime data,
and does not claim full Rust-only runtime.

## Decision

Decision A: Rust process ownership canary planning was added.

The new mode is intentionally non-default and non-mutating in DIFF-159. It
models a future bounded process canary without taking production worker
ownership.

## Runtime Mode Added

`igy6-worker` now supports:

```bash
IGY6_WORKER_PROCESS_CANARY=DIFF-159 igy6-worker --canary-loop --max-jobs N --max-idle-polls N [--claim-limit N] [--poll-interval-ms MS]
```

The default mode remains `--check`, which is safe and non-mutating.

`--canary-loop` renders a structured process-canary plan with:

- bounded `max_jobs`
- bounded `max_idle_polls`
- bounded `claim_limit`
- bounded `poll_interval_ms`
- stop conditions
- safety gates
- planned side effects for a later live process canary
- scheduler/beat posture

DIFF-159 does not make `--canary-loop` execute live queue claims. It does not
connect to PostgreSQL, read artifacts, write audit events, call Qdrant, control
Celery, or replace `beat`.

## Safety Gates

The process canary plan requires:

- `--canary-loop`
- `IGY6_WORKER_PROCESS_CANARY=DIFF-159`
- bounded `--max-jobs`
- bounded `--max-idle-polls`
- bounded `--claim-limit`
- bounded `--poll-interval-ms`
- Python/Celery worker remains production owner during this planning phase

The loop plan stops on:

- `max_jobs` reached
- `max_idle_polls` reached
- fatal validation error
- external shutdown signal requested

## Scheduler And Beat Posture

Scheduler/beat replacement is deferred.

DIFF-141 found no repo-defined beat schedule, but DIFF-159 does not remove or
replace `beat`. Docker Compose `beat` remains active until a later DIFF either
implements a Rust scheduler posture or formally retires scheduled work.

## Remaining Blockers

Before Python/Celery worker removal:

- Implement a live `--canary-loop` executor that claims queued supported work
  items without a named canary work item.
- Prove bounded repeated processing against isolated synthetic data.
- Prove graceful shutdown behavior and in-flight job handling.
- Prove retry/backoff behavior.
- Add a Rust worker Dockerfile or Compose canary service.
- Prove side-by-side canary operation without racing Python/Celery.
- Decide scheduler/beat replacement or retirement.
- Document rollback from Rust worker ownership back to Python/Celery.

## Current Runtime Posture

IGY6 remains Rust-primary with a Rust-native API path and retained
Python/Celery worker and beat services. Rust-only runtime is not claimed.

## Next Recommended DIFF

DIFF-160 should implement and run a bounded live Rust `--canary-loop` against an
isolated synthetic queue, with Python/Celery isolated from that queue, and
record observed repeated claim/execution/shutdown behavior.
