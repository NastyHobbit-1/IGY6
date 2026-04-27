from datetime import datetime
from typing import Any

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.db import get_db
from app.models import Hypothesis, Pattern, Prediction, Recommendation

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


@router.get("/patterns", response_model=list[PatternRead])
def list_patterns(db: Session = Depends(get_db)) -> list[Pattern]:
    statement = select(Pattern).order_by(Pattern.created_at.desc())
    return list(db.scalars(statement).all())


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


@router.get("/recommendations/{recommendation_id}", response_model=RecommendationRead)
def get_recommendation(recommendation_id: str, db: Session = Depends(get_db)) -> Recommendation:
    recommendation = db.get(Recommendation, recommendation_id)
    if recommendation is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Recommendation not found")
    return recommendation
