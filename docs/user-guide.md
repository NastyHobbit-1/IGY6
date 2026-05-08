# User Guide

The local web UI provides a read-only operational inventory.

Open:

```text
http://127.0.0.1:3000
```

The page shows API readiness checks, source records, collection-run metadata,
raw artifact metadata, normalized documents, chunks, evidence items, and claims
from FastAPI. It also shows read-only vector memory status, graph schema status,
existing pattern, hypothesis, prediction, and recommendation records, and a
retrieval-only chat preview. The preview submits a message to FastAPI and shows
retrieved context with `answer_status: not_generated`. It does not create
sources, run collection, upload files, normalize content, generate chunks,
create evidence, approve actions, generate chat answers, persist conversations,
predict, recommend, sync graph lineage, upsert vectors, or run experiments.

Future phases will add chat/review, pattern review, prediction/outcome tracking,
reports, and self-improvement controls.
