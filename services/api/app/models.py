from datetime import datetime

from sqlalchemy import Boolean, DateTime, ForeignKey, Integer, String, Text, func
from sqlalchemy.dialects.postgresql import JSONB
from sqlalchemy.orm import Mapped, mapped_column, relationship

from app.db import Base


class TimestampMixin:
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), nullable=False
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), onupdate=func.now(), nullable=False
    )


class Source(TimestampMixin, Base):
    __tablename__ = "sources"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    name: Mapped[str] = mapped_column(String(255), nullable=False)
    source_type: Mapped[str] = mapped_column(String(64), nullable=False)
    location: Mapped[str | None] = mapped_column(Text)
    owner_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    sensitivity: Mapped[str] = mapped_column(String(64), nullable=False, default="internal")
    trust_level: Mapped[str] = mapped_column(String(64), nullable=False, default="unreviewed")
    enabled: Mapped[bool] = mapped_column(Boolean, nullable=False, default=True)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)

    permissions: Mapped[list["SourcePermission"]] = relationship(back_populates="source")


class SourcePermission(TimestampMixin, Base):
    __tablename__ = "source_permissions"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    source_id: Mapped[str] = mapped_column(ForeignKey("sources.id"), nullable=False)
    scope_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    allowed_operations: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    external_model_policy: Mapped[str] = mapped_column(String(64), nullable=False, default="blocked")
    approval_required: Mapped[bool] = mapped_column(Boolean, nullable=False, default=True)
    created_by_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")

    source: Mapped[Source] = relationship(back_populates="permissions")


class WorkItem(TimestampMixin, Base):
    __tablename__ = "work_items"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    work_type: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="queued")
    requested_by_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    payload_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    error_message: Mapped[str | None] = mapped_column(Text)


class Approval(TimestampMixin, Base):
    __tablename__ = "approvals"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    request_type: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="pending")
    requested_by_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    decided_by_actor_id: Mapped[str | None] = mapped_column(String(128))
    decision_reason: Mapped[str | None] = mapped_column(Text)
    request_payload_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    decided_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))


class AuditEvent(Base):
    __tablename__ = "audit_events"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    created_at: Mapped[datetime] = mapped_column(
        DateTime(timezone=True), server_default=func.now(), nullable=False
    )
    actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="system")
    event_type: Mapped[str] = mapped_column(String(128), nullable=False)
    decision: Mapped[str | None] = mapped_column(String(64))
    resource_type: Mapped[str | None] = mapped_column(String(64))
    resource_id: Mapped[str | None] = mapped_column(String(128))
    correlation_id: Mapped[str | None] = mapped_column(String(128))
    details_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class CollectionRun(TimestampMixin, Base):
    __tablename__ = "collection_runs"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    source_id: Mapped[str | None] = mapped_column(ForeignKey("sources.id"))
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="created")
    dry_run: Mapped[bool] = mapped_column(Boolean, nullable=False, default=True)
    requested_by_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    summary_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    error_message: Mapped[str | None] = mapped_column(Text)


class RawArtifact(TimestampMixin, Base):
    __tablename__ = "raw_artifacts"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    source_id: Mapped[str | None] = mapped_column(ForeignKey("sources.id"))
    collection_run_id: Mapped[str | None] = mapped_column(ForeignKey("collection_runs.id"))
    content_hash: Mapped[str] = mapped_column(String(128), nullable=False)
    storage_path: Mapped[str] = mapped_column(Text, nullable=False)
    mime_type: Mapped[str | None] = mapped_column(String(255))
    size_bytes: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class NormalizedDocument(TimestampMixin, Base):
    __tablename__ = "normalized_documents"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    raw_artifact_id: Mapped[str | None] = mapped_column(ForeignKey("raw_artifacts.id"))
    source_id: Mapped[str | None] = mapped_column(ForeignKey("sources.id"))
    title: Mapped[str | None] = mapped_column(String(255))
    document_type: Mapped[str] = mapped_column(String(64), nullable=False, default="unknown")
    language: Mapped[str | None] = mapped_column(String(32))
    text_content: Mapped[str] = mapped_column(Text, nullable=False)
    sensitivity: Mapped[str] = mapped_column(String(64), nullable=False, default="internal")
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Chunk(TimestampMixin, Base):
    __tablename__ = "chunks"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    document_id: Mapped[str] = mapped_column(ForeignKey("normalized_documents.id"), nullable=False)
    chunk_index: Mapped[int] = mapped_column(Integer, nullable=False)
    text_content: Mapped[str] = mapped_column(Text, nullable=False)
    location_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    embedding_status: Mapped[str] = mapped_column(String(64), nullable=False, default="not_started")
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class EvidenceItem(TimestampMixin, Base):
    __tablename__ = "evidence_items"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    source_id: Mapped[str | None] = mapped_column(ForeignKey("sources.id"))
    document_id: Mapped[str | None] = mapped_column(ForeignKey("normalized_documents.id"))
    chunk_id: Mapped[str | None] = mapped_column(ForeignKey("chunks.id"))
    evidence_type: Mapped[str] = mapped_column(String(64), nullable=False)
    statement: Mapped[str] = mapped_column(Text, nullable=False)
    observed_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    confidence: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Claim(TimestampMixin, Base):
    __tablename__ = "claims"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    claim_text: Mapped[str] = mapped_column(Text, nullable=False)
    claim_type: Mapped[str] = mapped_column(String(64), nullable=False, default="statement")
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="unreviewed")
    evidence_ids: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    confidence: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Pattern(TimestampMixin, Base):
    __tablename__ = "patterns"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    pattern_type: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="candidate")
    summary: Mapped[str] = mapped_column(Text, nullable=False)
    evidence_ids: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    confidence: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Hypothesis(TimestampMixin, Base):
    __tablename__ = "hypotheses"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    hypothesis_text: Mapped[str] = mapped_column(Text, nullable=False)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="candidate")
    supporting_evidence_ids: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    missing_evidence_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    confidence: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Prediction(TimestampMixin, Base):
    __tablename__ = "predictions"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    prediction_text: Mapped[str] = mapped_column(Text, nullable=False)
    expected_result: Mapped[str] = mapped_column(Text, nullable=False)
    disproof_condition: Mapped[str | None] = mapped_column(Text)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="open")
    evidence_ids: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    confidence: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Recommendation(TimestampMixin, Base):
    __tablename__ = "recommendations"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    recommendation_text: Mapped[str] = mapped_column(Text, nullable=False)
    risk_level: Mapped[str] = mapped_column(String(64), nullable=False, default="unknown")
    approval_required: Mapped[bool] = mapped_column(Boolean, nullable=False, default=True)
    expected_result: Mapped[str | None] = mapped_column(Text)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="proposed")
    evidence_ids: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    confidence: Mapped[int | None] = mapped_column(Integer)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Outcome(TimestampMixin, Base):
    __tablename__ = "outcomes"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    target_type: Mapped[str] = mapped_column(String(64), nullable=False)
    target_id: Mapped[str] = mapped_column(String(36), nullable=False)
    outcome_status: Mapped[str] = mapped_column(String(64), nullable=False)
    summary: Mapped[str | None] = mapped_column(Text)
    occurred_at: Mapped[datetime | None] = mapped_column(DateTime(timezone=True))
    evidence_ids: Mapped[list[str]] = mapped_column(JSONB, nullable=False, default=list)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class FeedbackEvent(TimestampMixin, Base):
    __tablename__ = "feedback_events"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    target_type: Mapped[str] = mapped_column(String(64), nullable=False)
    target_id: Mapped[str] = mapped_column(String(36), nullable=False)
    label: Mapped[str] = mapped_column(String(64), nullable=False)
    actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    note: Mapped[str | None] = mapped_column(Text)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class ImprovementItem(TimestampMixin, Base):
    __tablename__ = "improvement_items"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    target_area: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="proposed")
    objective: Mapped[str] = mapped_column(Text, nullable=False)
    proposed_by_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    priority: Mapped[str] = mapped_column(String(64), nullable=False, default="normal")
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class ExperimentRun(TimestampMixin, Base):
    __tablename__ = "experiment_runs"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    improvement_item_id: Mapped[str | None] = mapped_column(ForeignKey("improvement_items.id"))
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="planned")
    mlflow_run_id: Mapped[str | None] = mapped_column(String(255))
    optuna_study_name: Mapped[str | None] = mapped_column(String(255))
    metrics_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    artifacts_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)


class Report(TimestampMixin, Base):
    __tablename__ = "reports"

    id: Mapped[str] = mapped_column(String(36), primary_key=True)
    title: Mapped[str] = mapped_column(String(255), nullable=False)
    report_type: Mapped[str] = mapped_column(String(64), nullable=False)
    status: Mapped[str] = mapped_column(String(64), nullable=False, default="placeholder")
    requested_by_actor_id: Mapped[str] = mapped_column(String(128), nullable=False, default="local-owner")
    artifact_path: Mapped[str | None] = mapped_column(Text)
    metadata_json: Mapped[dict] = mapped_column(JSONB, nullable=False, default=dict)
