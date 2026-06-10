# Architecture

IGY6 is a local-first evidence platform:

- Rust gateway and API
- Rust worker daemon for background processing
- Next.js web UI
- Supporting services: PostgreSQL, Qdrant (vector), Neo4j (graph), MLflow, Phoenix

Evidence pipeline processes artifacts into documents, chunks, vector memory, graph context, answers, and full provenance.