from datetime import datetime
from collections import defaultdict
from typing import Any
from uuid import uuid4

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import AuditEvent, EvidenceItem, Hypothesis, Pattern, Prediction, Recommendation

router = APIRouter(prefix="/analysis", tags=["analysis"])


class PatternRead(BaseModel):
    id: str
    pattern_type: str
    status: str
    summary: str
    evidence_ids: list[str] = Field(default_factory=list)
    confidence: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class PatternCreate(BaseModel):
    pattern_type: str = Field(min_length=1, max_length=64)
    summary: str = Field(min_length=1)
    evidence_ids: list[str] = Field(min_length=1)
    confidence: int | None = Field(default=None, ge=0, le=100)
    status: str = Field(default="candidate", min_length=1, max_length=64)
    actor_id: str = "local-owner"
    metadata_json: dict[str, Any] = Field(default_factory=dict)


class PatternReview(BaseModel):
    status: str = Field(min_length=1, max_length=64)
    reviewed_by_actor_id: str = "local-owner"
    review_note: str | None = None


class BaselinePatternDetectRequest(BaseModel):
    actor_id: str = "local-owner"
    recurrence_threshold: int = Field(default=3, ge=2, le=20)


class HypothesisRead(BaseModel):
    id: str
    hypothesis_text: str
    status: str
    supporting_evidence_ids: list[str] = Field(default_factory=list)
    missing_evidence_json: dict[str, Any]
    confidence: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class HypothesisCreate(BaseModel):
    hypothesis_text: str = Field(min_length=1)
    supporting_evidence_ids: list[str] = Field(min_length=1)
    missing_evidence_json: dict[str, Any] = Field(default_factory=dict)
    confidence: int | None = Field(default=None, ge=0, le=100)
    status: str = Field(default="candidate", min_length=1, max_length=64)
    actor_id: str = "local-owner"
    metadata_json: dict[str, Any] = Field(default_factory=dict)


class PredictionRead(BaseModel):
    id: str
    prediction_text: str
    expected_result: str
    disproof_condition: str | None
    status: str
    evidence_ids: list[str] = Field(default_factory=list)
    confidence: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class PredictionCreate(BaseModel):
    prediction_text: str = Field(min_length=1)
    expected_result: str = Field(min_length=1)
    disproof_condition: str | None = None
    evidence_ids: list[str] = Field(min_length=1)
    confidence: int | None = Field(default=None, ge=0, le=100)
    status: str = Field(default="open", min_length=1, max_length=64)
    actor_id: str = "local-owner"
    metadata_json: dict[str, Any] = Field(default_factory=dict)


class RecommendationRead(BaseModel):
    id: str
    recommendation_text: str
    risk_level: str
    approval_required: bool
    expected_result: str | None
    status: str
    evidence_ids: list[str] = Field(default_factory=list)
    confidence: int | None
    metadata_json: dict[str, Any]
    created_at: datetime
    updated_at: datetime

    model_config = {"from_attributes": True}


class RecommendationCreate(BaseModel):
    recommendation_text: str = Field(min_length=1)
    risk_level: str = Field(default="unknown", min_length=1, max_length=64)
    approval_required: bool = True
    expected_result: str | None = None
    evidence_ids: list[str] = Field(min_length=1)
    confidence: int | None = Field(default=None, ge=0, le=100)
    status: str = Field(default="proposed", min_length=1, max_length=64)
    actor_id: str = "local-owner"
    metadata_json: dict[str, Any] = Field(default_factory=dict)


def _validated_evidence_ids(db: Session, evidence_ids: list[str]) -> list[str]:
    unique_ids = list(dict.fromkeys(evidence_ids))
    statement = select(EvidenceItem.id).where(EvidenceItem.id.in_(unique_ids))
    found_ids = set(db.scalars(statement).all())
    missing_ids = [evidence_id for evidence_id in unique_ids if evidence_id not in found_ids]
    if missing_ids:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail={
                "message": "Analysis records must reference existing evidence items",
                "missing_evidence_ids": missing_ids,
            },
        )
    return unique_ids


def _audit_analysis_created(
    db: Session,
    *,
    actor_id: str,
    resource_type: str,
    resource_id: str,
    evidence_ids: list[str],
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type=f"analysis.{resource_type}.created",
            decision="recorded",
            resource_type=resource_type,
            resource_id=resource_id,
            correlation_id=None,
            details_json={
                "evidence_ids": evidence_ids,
            },
        )
    )


def _audit_pattern_reviewed(
    db: Session,
    *,
    actor_id: str,
    pattern: Pattern,
    previous_status: str,
    review_note: str | None,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="analysis.pattern.reviewed",
            decision=pattern.status,
            resource_type="pattern",
            resource_id=pattern.id,
            correlation_id=None,
            details_json={
                "previous_status": previous_status,
                "new_status": pattern.status,
                "review_note": review_note,
            },
        )
    )


def _normalize_statement(value: str) -> str:
    return " ".join(value.lower().split())[:240]


def baseline_pattern_candidates(
    evidence_items: list[EvidenceItem],
    *,
    recurrence_threshold: int,
) -> list[dict[str, Any]]:
    if not evidence_items:
        return [
            {
                "pattern_type": "missing_information_gap",
                "summary": "No evidence items exist yet, so the system cannot detect grounded patterns.",
                "evidence_ids": [],
                "confidence": 100,
                "detector_key": "missing_information_gap:no_evidence",
            }
        ]

    candidates: list[dict[str, Any]] = []
    by_type: dict[str, list[EvidenceItem]] = defaultdict(list)
    by_statement: dict[str, list[EvidenceItem]] = defaultdict(list)
    for item in evidence_items:
        by_type[item.evidence_type].append(item)
        by_statement[_normalize_statement(item.statement)].append(item)

    for evidence_type, items in sorted(by_type.items()):
        if len(items) >= recurrence_threshold:
            candidates.append(
                {
                    "pattern_type": "recurrence",
                    "summary": f"{len(items)} evidence items share evidence type `{evidence_type}`.",
                    "evidence_ids": [item.id for item in items[:10]],
                    "confidence": min(90, 50 + len(items) * 5),
                    "detector_key": f"recurrence:evidence_type:{evidence_type}",
                }
            )

    for normalized_statement, items in sorted(by_statement.items()):
        source_ids = {item.source_id for item in items if item.source_id is not None}
        if len(source_ids) >= 2:
            candidates.append(
                {
                    "pattern_type": "cross_source_conflict",
                    "summary": (
                        "Multiple sources contain the same normalized evidence statement; "
                        "review whether they agree, duplicate, or conflict."
                    ),
                    "evidence_ids": [item.id for item in items[:10]],
                    "confidence": 60,
                    "detector_key": f"cross_source_statement:{normalized_statement}",
                }
            )

    return candidates


def _existing_detector_keys(db: Session) -> set[str]:
    rows = db.scalars(select(Pattern)).all()
    return {
        pattern.metadata_json.get("detector_key")
        for pattern in rows
        if isinstance(pattern.metadata_json, dict) and pattern.metadata_json.get("detector_key")
    }


@router.get("/patterns", response_model=list[PatternRead])
def list_patterns(db: Session = Depends(get_db)) -> list[Pattern]:
    statement = select(Pattern).order_by(Pattern.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("/patterns", response_model=PatternRead, status_code=status.HTTP_201_CREATED)
def create_pattern(payload: PatternCreate, db: Session = Depends(get_db)) -> Pattern:
    evidence_ids = _validated_evidence_ids(db, payload.evidence_ids)
    pattern = Pattern(
        id=str(uuid4()),
        pattern_type=payload.pattern_type,
        status=payload.status,
        summary=payload.summary,
        evidence_ids=evidence_ids,
        confidence=payload.confidence,
        metadata_json=payload.metadata_json,
    )
    db.add(pattern)
    _audit_analysis_created(
        db,
        actor_id=payload.actor_id,
        resource_type="pattern",
        resource_id=pattern.id,
        evidence_ids=evidence_ids,
    )
    db.commit()
    db.refresh(pattern)
    return pattern


@router.post("/patterns/{pattern_id}/review", response_model=PatternRead)
def review_pattern(
    pattern_id: str,
    payload: PatternReview,
    db: Session = Depends(get_db),
) -> Pattern:
    pattern = db.get(Pattern, pattern_id)
    if pattern is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Pattern not found")
    if payload.status not in {"verified", "rejected"}:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail="Pattern review status must be verified or rejected",
        )
    if pattern.status != "candidate":
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Only candidate patterns can be reviewed",
        )

    previous_status = pattern.status
    pattern.status = payload.status
    _audit_pattern_reviewed(
        db,
        actor_id=payload.reviewed_by_actor_id,
        pattern=pattern,
        previous_status=previous_status,
        review_note=payload.review_note,
    )
    db.commit()
    db.refresh(pattern)
    return pattern


@router.post("/patterns/detect-baseline", response_model=list[PatternRead], status_code=status.HTTP_201_CREATED)
def detect_baseline_patterns(
    payload: BaselinePatternDetectRequest,
    db: Session = Depends(get_db),
) -> list[Pattern]:
    evidence_items = list(db.scalars(select(EvidenceItem).order_by(EvidenceItem.created_at.desc())).all())
    existing_keys = _existing_detector_keys(db)
    created_patterns: list[Pattern] = []
    for candidate in baseline_pattern_candidates(evidence_items, recurrence_threshold=payload.recurrence_threshold):
        detector_key = candidate["detector_key"]
        if detector_key in existing_keys:
            continue
        pattern = Pattern(
            id=str(uuid4()),
            pattern_type=candidate["pattern_type"],
            status="candidate",
            summary=candidate["summary"],
            evidence_ids=candidate["evidence_ids"],
            confidence=candidate["confidence"],
            metadata_json={
                "generated_by": "DIFF-069",
                "detector": "baseline_local_v1",
                "detector_key": detector_key,
            },
        )
        db.add(pattern)
        _audit_analysis_created(
            db,
            actor_id=payload.actor_id,
            resource_type="pattern",
            resource_id=pattern.id,
            evidence_ids=pattern.evidence_ids,
        )
        created_patterns.append(pattern)
        existing_keys.add(detector_key)

    db.commit()
    for pattern in created_patterns:
        db.refresh(pattern)
    return created_patterns


@router.get("/patterns/{pattern_id}", response_model=PatternRead)
def get_pattern(pattern_id: str, db: Session = Depends(get_db)) -> Pattern:
    pattern = db.get(Pattern, pattern_id)
    if pattern is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Pattern not found")
    return pattern


@router.get("/hypotheses", response_model=list[HypothesisRead])
def list_hypotheses(db: Session = Depends(get_db)) -> list[Hypothesis]:
    statement = select(Hypothesis).order_by(Hypothesis.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("/hypotheses", response_model=HypothesisRead, status_code=status.HTTP_201_CREATED)
def create_hypothesis(payload: HypothesisCreate, db: Session = Depends(get_db)) -> Hypothesis:
    evidence_ids = _validated_evidence_ids(db, payload.supporting_evidence_ids)
    hypothesis = Hypothesis(
        id=str(uuid4()),
        hypothesis_text=payload.hypothesis_text,
        status=payload.status,
        supporting_evidence_ids=evidence_ids,
        missing_evidence_json=payload.missing_evidence_json,
        confidence=payload.confidence,
        metadata_json=payload.metadata_json,
    )
    db.add(hypothesis)
    _audit_analysis_created(
        db,
        actor_id=payload.actor_id,
        resource_type="hypothesis",
        resource_id=hypothesis.id,
        evidence_ids=evidence_ids,
    )
    db.commit()
    db.refresh(hypothesis)
    return hypothesis


@router.get("/hypotheses/{hypothesis_id}", response_model=HypothesisRead)
def get_hypothesis(hypothesis_id: str, db: Session = Depends(get_db)) -> Hypothesis:
    hypothesis = db.get(Hypothesis, hypothesis_id)
    if hypothesis is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Hypothesis not found")
    return hypothesis


@router.get("/predictions", response_model=list[PredictionRead])
def list_predictions(db: Session = Depends(get_db)) -> list[Prediction]:
    statement = select(Prediction).order_by(Prediction.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("/predictions", response_model=PredictionRead, status_code=status.HTTP_201_CREATED)
def create_prediction(payload: PredictionCreate, db: Session = Depends(get_db)) -> Prediction:
    evidence_ids = _validated_evidence_ids(db, payload.evidence_ids)
    prediction = Prediction(
        id=str(uuid4()),
        prediction_text=payload.prediction_text,
        expected_result=payload.expected_result,
        disproof_condition=payload.disproof_condition,
        status=payload.status,
        evidence_ids=evidence_ids,
        confidence=payload.confidence,
        metadata_json=payload.metadata_json,
    )
    db.add(prediction)
    _audit_analysis_created(
        db,
        actor_id=payload.actor_id,
        resource_type="prediction",
        resource_id=prediction.id,
        evidence_ids=evidence_ids,
    )
    db.commit()
    db.refresh(prediction)
    return prediction


@router.get("/predictions/{prediction_id}", response_model=PredictionRead)
def get_prediction(prediction_id: str, db: Session = Depends(get_db)) -> Prediction:
    prediction = db.get(Prediction, prediction_id)
    if prediction is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Prediction not found")
    return prediction


@router.get("/recommendations", response_model=list[RecommendationRead])
def list_recommendations(db: Session = Depends(get_db)) -> list[Recommendation]:
    statement = select(Recommendation).order_by(Recommendation.created_at.desc())
    return list(db.scalars(statement).all())


@router.post("/recommendations", response_model=RecommendationRead, status_code=status.HTTP_201_CREATED)
def create_recommendation(
    payload: RecommendationCreate,
    db: Session = Depends(get_db),
) -> Recommendation:
    evidence_ids = _validated_evidence_ids(db, payload.evidence_ids)
    recommendation = Recommendation(
        id=str(uuid4()),
        recommendation_text=payload.recommendation_text,
        risk_level=payload.risk_level,
        approval_required=payload.approval_required,
        expected_result=payload.expected_result,
        status=payload.status,
        evidence_ids=evidence_ids,
        confidence=payload.confidence,
        metadata_json=payload.metadata_json,
    )
    db.add(recommendation)
    _audit_analysis_created(
        db,
        actor_id=payload.actor_id,
        resource_type="recommendation",
        resource_id=recommendation.id,
        evidence_ids=evidence_ids,
    )
    db.commit()
    db.refresh(recommendation)
    return recommendation


@router.get("/recommendations/{recommendation_id}", response_model=RecommendationRead)
def get_recommendation(recommendation_id: str, db: Session = Depends(get_db)) -> Recommendation:
    recommendation = db.get(Recommendation, recommendation_id)
    if recommendation is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Recommendation not found")
    return recommendation
