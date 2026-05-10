from functools import lru_cache

from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    celery_broker_url: str = "redis://redis:6379/0"
    celery_result_backend: str = "redis://redis:6379/1"
    database_url: str = "postgresql+psycopg://adaptive:change-me-local-only@postgres:5432/adaptive_intelligence"
    artifact_store_path: str = "/workspace/storage/artifacts"
    qdrant_url: str = "http://qdrant:6333"
    qdrant_chunk_collection: str = "igy6_chunks"
    qdrant_chunk_vector_size: int = 384


@lru_cache
def get_settings() -> Settings:
    return Settings()
