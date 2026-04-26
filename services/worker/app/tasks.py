from app.celery_app import celery_app


@celery_app.task(name="phase0.health")
def health() -> dict[str, str]:
    return {"status": "ok", "service": "worker", "phase": "0"}
