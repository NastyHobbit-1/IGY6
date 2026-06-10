# Architecture

IGY6 is a local-first evidence and intelligence workspace. The runtime stack is designed to keep application data under local operator control while providing a web UI, Rust services, and local supporting databases.

## Runtime components

| Component | Purpose |
|---|---|
| Next.js web UI | Browser interface for chat, data review, work status, settings, and diagnostics |
| Rust gateway/API | Primary local API surface |
| Rust worker daemon | Background processing for queued runtime work |
| PostgreSQL | Structured runtime metadata and records |
| Qdrant | Vector memory and similarity search |
| Neo4j | Graph relationships, lineage, and context |
| MLflow | Local experiment and run tracking when enabled |
| Phoenix | Local observability support when enabled |

## Evidence pipeline

Runtime evidence moves through a structured local pipeline:

1. Artifacts are registered under the configured runtime data root.
2. Documents are normalized from authorized source material.
3. Documents are chunked into bounded evidence units.
4. Chunks are indexed into vector memory.
5. Relationships and lineage can be represented as graph context.
6. Evidence-grounded answers reference retrieved material.
7. Provenance trails connect answers back to source evidence.

## Runtime data location

Runtime data belongs outside the repository under `IGY6_DATA_ROOT`.

Do not commit:

- `.env`
- `.env.test`
- runtime storage
- logs
- database files
- exports
- generated artifacts
- caches
