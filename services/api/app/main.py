from fastapi import FastAPI

from app.approvals import router as approvals_router
from app.health import router as health_router
from app.sources import router as sources_router
from app.work_items import router as work_items_router

app = FastAPI(
    title="IGY6 Adaptive Intelligence API",
    version="0.0.0-phase0",
    description="Phase 0 local-first skeleton. No ingestion, chat, prediction, or experiments.",
)

app.include_router(health_router)
app.include_router(sources_router)
app.include_router(work_items_router)
app.include_router(approvals_router)


@app.get("/")
def root() -> dict[str, str]:
    return {
        "service": "igy6-api",
        "phase": "0",
        "status": "skeleton",
    }
