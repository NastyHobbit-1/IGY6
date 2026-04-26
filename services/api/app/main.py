from fastapi import FastAPI

from app.health import router as health_router

app = FastAPI(
    title="IGY6 Adaptive Intelligence API",
    version="0.0.0-phase0",
    description="Phase 0 local-first skeleton. No ingestion, chat, prediction, or experiments.",
)

app.include_router(health_router)


@app.get("/")
def root() -> dict[str, str]:
    return {
        "service": "igy6-api",
        "phase": "0",
        "status": "skeleton",
    }
