# Migrations

The legacy Alembic migration history was archived with the legacy FastAPI API
source in `archive/legacy-python/services-api/migrations` during DIFF-139.

The active Docker Compose API service is the Rust gateway and does not run
Alembic on startup. Database schema ownership remains a follow-up migration
governance item; do not delete the archived migration history.

This directory is reserved for migration operations notes and cross-service
database guidance.
