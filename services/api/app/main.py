from fastapi import FastAPI

from app.analysis import router as analysis_router
from app.collection_runs import router as collection_runs_router
from app.approvals import router as approvals_router
from app.audit import router as audit_router
from app.artifacts import router as artifacts_router
from app.evidence import router as evidence_router
from app.feedback import router as feedback_router
from app.health import router as health_router
from app.outcomes import router as outcomes_router
from app.reports import router as reports_router
from app.sources import router as sources_router
from app.work_items import router as work_items_router

app = FastAPI(
    title="IGY6 Adaptive Intelligence API",
    version="0.1.0-phase1-foundation",
    description=(
        "Local-first Phase 1 foundation with health, source registry, work item intent, "
        "and approval record APIs. No collection, chat, prediction, or experiments."
    ),
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


@app.get("/")
def root() -> dict[str, str]:
    return {
        "service": "igy6-api",
        "phase": "1",
        "status": "foundation",
    }
