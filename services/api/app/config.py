from functools import lru_cache

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class Settings(BaseSettings):
    model_config = SettingsConfigDict(env_file=".env", extra="ignore")

    app_env: str = "local"
    app_host: str = "127.0.0.1"
    app_port: int = 8000
    single_user_mode: bool = True
    approval_required_default: bool = True
    external_model_policy_default: str = "blocked"

    database_url: str = Field(
        default="postgresql+psycopg://adaptive:change-me-local-only@postgres:5432/adaptive_intelligence"
    )
    redis_url: str = "redis://redis:6379/0"
    qdrant_url: str = "http://qdrant:6333"
    neo4j_uri: str = "bolt://neo4j:7687"
    neo4j_user: str = "neo4j"
    neo4j_password: str = "change-me-local-only"
    mlflow_tracking_uri: str = "http://mlflow:5000"
    phoenix_collector_endpoint: str = "http://phoenix:6006"
    artifact_store_path: str = "/workspace/storage/artifacts"
    export_store_path: str = "/workspace/storage/exports"


@lru_cache
def get_settings() -> Settings:
    return Settings()
