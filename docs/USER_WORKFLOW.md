# User Workflow

IGY6 is operated as a local evidence workspace through the web UI and local API gateway.

## Normal workflow

1. Start the local stack with Docker Compose.
2. Open the web UI at `http://127.0.0.1:13000`.
3. Add authorized information through the available data/source UI.
4. Let the runtime register artifacts and process documents.
5. Review available evidence and provenance trails.
6. Ask evidence-grounded questions through the chat interface.
7. Review answer support, source references, and confidence indicators.
8. Monitor work items and reports when those runtime areas are available.
9. Stop or restart the stack through Docker Compose when finished.

## Evidence review

Evidence-grounded answers should be checked against their supporting material. Provenance trails are used to understand where a claim came from and which source material contributed to the answer.

## Local data handling

Keep runtime data outside the repository under `IGY6_DATA_ROOT`.

Do not commit `.env`, `.env.test`, logs, database files, exports, generated artifacts, caches, or runtime storage.
