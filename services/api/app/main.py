from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

from app.analysis import router as analysis_router
from app.collection_runs import router as collection_runs_router
from app.approvals import router as approvals_router
from app.audit import router as audit_router
from app.artifacts import router as artifacts_router
from app.chat import router as chat_router
from app.evidence import router as evidence_router
from app.experiments import router as experiments_router
from app.feedback import router as feedback_router
from app.graph_memory import router as graph_memory_router
from app.health import router as health_router
from app.improvements import router as improvements_router
from app.outcomes import router as outcomes_router
from app.reports import router as reports_router
from app.retrieval import router as retrieval_router
from app.settings_env import router as settings_env_router
from app.sources import router as sources_router
from app.vector_memory import router as vector_memory_router
from app.work_items import router as work_items_router

app = FastAPI(
    title="IGY6 Adaptive Intelligence API",
    version="0.1.0-memory-review-foundation",
    description=(
        "Local-first foundation with source, evidence, collection, vector, graph, "
        "review, worker, improvement, and experiment metadata APIs. Answer generation, "
        "self-improvement execution, and production method changes are not implemented."
    ),
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=[
        "http://127.0.0.1:3000",
        "http://127.0.0.1:3001",
        "http://localhost:3000",
        "http://localhost:3001",
    ],
    allow_credentials=False,
    allow_methods=["GET", "POST", "OPTIONS"],
    allow_headers=["content-type"],
)

app.include_router(health_router)
app.include_router(sources_router)
app.include_router(work_items_router)
app.include_router(approvals_router)
app.include_router(evidence_router)
app.include_router(feedback_router)
app.include_router(outcomes_router)
app.include_router(reports_router)
app.include_router(analysis_router)
app.include_router(audit_router)
app.include_router(artifacts_router)
app.include_router(collection_runs_router)
app.include_router(retrieval_router)
app.include_router(vector_memory_router)
app.include_router(graph_memory_router)
app.include_router(chat_router)
app.include_router(improvements_router)
app.include_router(experiments_router)
app.include_router(settings_env_router)


@app.get("/")
def root() -> dict[str, str]:
    return {
        "service": "igy6-api",
        "phase": "memory-review-foundation",
        "status": "scaffolded",
    }
