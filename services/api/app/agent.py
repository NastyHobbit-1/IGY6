from fastapi import APIRouter, Depends
from sqlalchemy.orm import Session

from app.agent_actions import (
    AgentActionExecuteRequest,
    AgentActionExecuteResponse,
    AgentCapabilitiesResponse,
    AgentIntentRequest,
    AgentIntentResponse,
    classify_agent_intent,
    execute_agent_action,
    get_agent_capabilities,
)
from app.config import Settings, get_settings
from app.db import get_db

router = APIRouter(prefix="/agent", tags=["agent"])


@router.get("/capabilities", response_model=AgentCapabilitiesResponse)
def read_agent_capabilities() -> AgentCapabilitiesResponse:
    return get_agent_capabilities()


@router.post("/intent", response_model=AgentIntentResponse)
def create_agent_intent(payload: AgentIntentRequest) -> AgentIntentResponse:
    return classify_agent_intent(payload)


@router.post("/actions/{action_name}/execute", response_model=AgentActionExecuteResponse)
def execute_agent_action_endpoint(
    action_name: str,
    payload: AgentActionExecuteRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> AgentActionExecuteResponse:
    return execute_agent_action(action_name, payload, db=db, settings=settings)
