from __future__ import annotations

import subprocess
from dataclasses import dataclass
from datetime import UTC, datetime
from os import environ
from pathlib import Path
from shutil import which
from typing import Any, Literal

from fastapi import HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy import select
from sqlalchemy.orm import Session

from app.chat import ChatRetrievalPreviewRequest, build_retrieval_preview
from app.config import Settings
from app.health import ready
from app.models import Approval, AuditEvent, WorkItem


ActionName = Literal[
    "show_project_health",
    "show_git_status",
    "show_latest_diff",
    "show_work_items",
    "run_retrieval_preview",
    "start_stack",
    "stop_stack",
    "run_last_healthy_stack",
]


class AgentIntentRequest(BaseModel):
    message: str = Field(min_length=1)
    parameters: dict[str, Any] = Field(default_factory=dict)
    actor_id: str = "local-owner"


class AgentIntentResponse(BaseModel):
    original_message: str
    interpreted_intent: str
    proposed_action: str | None
    action_type: Literal["read_only", "system_changing", "unknown"]
    approval_required: bool
    risk_level: Literal["low", "medium", "high"]
    required_parameters: list[str]
    missing_parameters: list[str]
    safety_notes: list[str]
    executable_now: bool
    reason: str | None = None


class AgentActionExecuteRequest(BaseModel):
    parameters: dict[str, Any] = Field(default_factory=dict)
    approval_id: str | None = None
    actor_id: str = "local-owner"


class AgentActionExecuteResponse(BaseModel):
    action_name: str
    status: Literal["completed", "failed", "blocked"]
    result: dict[str, Any] = Field(default_factory=dict)
    stdout_summary: str | None = None
    stderr_summary: str | None = None
    exit_code: int | None = None
    started_at: datetime
    finished_at: datetime
    audit_event_id: int | None = None


class AgentRuntimeCapabilities(BaseModel):
    repo_root: str
    docker_cli_available: bool
    docker_compose_available: bool
    docker_socket_available: bool
    docker_host_configured: bool
    docker_control_available: bool
    docker_socket_path: str | None = None
    reason: str | None = None


class AgentActionCapability(BaseModel):
    name: str
    interpreted_intent: str
    action_type: Literal["read_only", "system_changing"]
    approval_required: bool
    risk_level: Literal["low", "medium", "high"]
    required_parameters: list[str]
    script_backed: bool
    required_scripts: list[str]
    scripts_exist: bool
    executable_in_api_runtime: bool
    reason: str | None = None


class AgentCapabilitiesResponse(BaseModel):
    actions: list[AgentActionCapability]
    runtime: AgentRuntimeCapabilities


@dataclass(frozen=True)
class AgentActionDefinition:
    name: str
    interpreted_intent: str
    action_type: Literal["read_only", "system_changing"]
    approval_required: bool
    risk_level: Literal["low", "medium", "high"]
    required_parameters: tuple[str, ...] = ()
    safety_notes: tuple[str, ...] = ()
    script_argv: tuple[str, ...] | None = None


ACTION_REGISTRY: dict[str, AgentActionDefinition] = {
    "show_project_health": AgentActionDefinition(
        name="show_project_health",
        interpreted_intent="Show local IGY6 API readiness and dependency health.",
        action_type="read_only",
        approval_required=False,
        risk_level="low",
        safety_notes=("Read-only local health check.",),
    ),
    "show_git_status": AgentActionDefinition(
        name="show_git_status",
        interpreted_intent="Show the current repository branch, commit, and dirty/clean state.",
        action_type="read_only",
        approval_required=False,
        risk_level="low",
        safety_notes=("Read-only git metadata only; no diff content or secrets are returned.",),
    ),
    "show_latest_diff": AgentActionDefinition(
        name="show_latest_diff",
        interpreted_intent="Show the newest DIFF document and status.",
        action_type="read_only",
        approval_required=False,
        risk_level="low",
        safety_notes=("Read-only DIFF metadata lookup.",),
    ),
    "show_work_items": AgentActionDefinition(
        name="show_work_items",
        interpreted_intent="Show recent local work items.",
        action_type="read_only",
        approval_required=False,
        risk_level="low",
        safety_notes=("Read-only PostgreSQL work-item metadata.",),
    ),
    "run_retrieval_preview": AgentActionDefinition(
        name="run_retrieval_preview",
        interpreted_intent="Run deterministic local retrieval preview.",
        action_type="read_only",
        approval_required=False,
        risk_level="low",
        required_parameters=("message",),
        safety_notes=("Uses existing retrieval preview; no LLM or external model is called.",),
    ),
    "start_stack": AgentActionDefinition(
        name="start_stack",
        interpreted_intent="Start the local IGY6 Docker Compose stack detached.",
        action_type="system_changing",
        approval_required=True,
        risk_level="high",
        safety_notes=("Requires approved approval record.", "Uses scripts/run.sh --detached only."),
        script_argv=("scripts/run.sh", "--detached"),
    ),
    "stop_stack": AgentActionDefinition(
        name="stop_stack",
        interpreted_intent="Stop the local IGY6 Docker Compose stack without deleting data.",
        action_type="system_changing",
        approval_required=True,
        risk_level="high",
        safety_notes=("Requires approved approval record.", "Uses scripts/stop.sh and preserves volumes/data."),
        script_argv=("scripts/stop.sh",),
    ),
    "run_last_healthy_stack": AgentActionDefinition(
        name="run_last_healthy_stack",
        interpreted_intent="Start from the last healthy local stack snapshot.",
        action_type="system_changing",
        approval_required=True,
        risk_level="high",
        safety_notes=("Requires approved approval record.", "Uses scripts/run-last-healthy-config.sh only."),
        script_argv=("scripts/run-last-healthy-config.sh",),
    ),
}


DANGEROUS_PATTERNS = (
    "rm -rf",
    "docker system prune",
    "docker volume rm",
    "git reset",
    "git checkout",
    "git stash",
    "bash -c",
    "sh -c",
    "powershell",
    "cmd.exe",
    "format ",
)


def repo_root() -> Path:
    candidates: list[Path] = []
    configured_env_path = environ.get("ENV_FILE_PATH")
    if configured_env_path:
        candidates.append(Path(configured_env_path).expanduser().resolve().parent)
    candidates.append(Path("/workspace/project"))
    current_file = Path(__file__).resolve()
    if len(current_file.parents) > 3:
        candidates.append(current_file.parents[3])
    for candidate in candidates:
        if (candidate / "AGENTS.md").is_file() and (candidate / "docs" / "diffs").is_dir():
            return candidate
    return candidates[-1] if candidates else current_file.parent


def classify_agent_intent(payload: AgentIntentRequest) -> AgentIntentResponse:
    message = payload.message.strip()
    normalized = " ".join(message.lower().split())
    action_name: str | None = None
    reason: str | None = None

    if any(pattern in normalized for pattern in DANGEROUS_PATTERNS):
        return _unknown_intent(
            message,
            "Arbitrary shell or destructive command requests are not allowed by the typed action registry.",
        )

    if "start" in normalized and "stack" in normalized:
        action_name = "start_stack"
    elif "stop" in normalized and "stack" in normalized:
        action_name = "stop_stack"
    elif "last healthy" in normalized or "last-known healthy" in normalized:
        action_name = "run_last_healthy_stack"
    elif "git" in normalized and ("status" in normalized or "state" in normalized):
        action_name = "show_git_status"
    elif "latest diff" in normalized or "newest diff" in normalized or "current diff" in normalized:
        action_name = "show_latest_diff"
    elif "work item" in normalized or "work queue" in normalized:
        action_name = "show_work_items"
    elif "retrieval preview" in normalized or "preview retrieval" in normalized or "preview context" in normalized:
        action_name = "run_retrieval_preview"
    elif "health" in normalized or "ready" in normalized or "readiness" in normalized:
        action_name = "show_project_health"
    else:
        reason = "No known local project action matched the message."

    if action_name is None:
        return _unknown_intent(message, reason or "Unknown action.")

    definition = ACTION_REGISTRY[action_name]
    missing = [
        parameter
        for parameter in definition.required_parameters
        if parameter not in payload.parameters or payload.parameters.get(parameter) in (None, "")
    ]
    return AgentIntentResponse(
        original_message=message,
        interpreted_intent=definition.interpreted_intent,
        proposed_action=definition.name,
        action_type=definition.action_type,
        approval_required=definition.approval_required,
        risk_level=definition.risk_level,
        required_parameters=list(definition.required_parameters),
        missing_parameters=missing,
        safety_notes=list(definition.safety_notes),
        executable_now=not missing and not definition.approval_required,
        reason="Approval required before execution." if definition.approval_required else None,
    )


def get_agent_capabilities() -> AgentCapabilitiesResponse:
    runtime = _runtime_capabilities()
    actions = []
    for definition in ACTION_REGISTRY.values():
        required_scripts = [definition.script_argv[0]] if definition.script_argv else []
        scripts_exist = all((repo_root() / script).is_file() for script in required_scripts)
        if definition.script_argv:
            executable = scripts_exist and runtime.docker_control_available
            reason = None if executable else _script_capability_reason(scripts_exist, runtime)
        else:
            executable = True
            reason = None
        actions.append(
            AgentActionCapability(
                name=definition.name,
                interpreted_intent=definition.interpreted_intent,
                action_type=definition.action_type,
                approval_required=definition.approval_required,
                risk_level=definition.risk_level,
                required_parameters=list(definition.required_parameters),
                script_backed=definition.script_argv is not None,
                required_scripts=required_scripts,
                scripts_exist=scripts_exist,
                executable_in_api_runtime=executable,
                reason=reason,
            )
        )
    return AgentCapabilitiesResponse(actions=actions, runtime=runtime)


def _unknown_intent(message: str, reason: str) -> AgentIntentResponse:
    return AgentIntentResponse(
        original_message=message,
        interpreted_intent="No allowed typed action was selected.",
        proposed_action=None,
        action_type="unknown",
        approval_required=False,
        risk_level="high",
        required_parameters=[],
        missing_parameters=[],
        safety_notes=[
            "The agent command plane only accepts fixed local project actions.",
            "Arbitrary shell execution is not available.",
        ],
        executable_now=False,
        reason=reason,
    )


def execute_agent_action(
    action_name: str,
    payload: AgentActionExecuteRequest,
    *,
    db: Session,
    settings: Settings,
) -> AgentActionExecuteResponse:
    definition = ACTION_REGISTRY.get(action_name)
    if definition is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Unknown agent action")

    missing = [
        parameter
        for parameter in definition.required_parameters
        if parameter not in payload.parameters or payload.parameters.get(parameter) in (None, "")
    ]
    if missing:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail={"message": "Missing required action parameters", "missing_parameters": missing},
        )

    if definition.approval_required:
        _require_action_approval(db, definition, payload)
    if definition.script_argv:
        _require_script_runtime_capability(definition)

    started_at = datetime.now(UTC)
    start_event = _audit_agent_action(
        db,
        actor_id=payload.actor_id,
        event_type="agent.action.started",
        decision="started",
        action_name=action_name,
        details={"parameters": _safe_parameter_summary(payload.parameters), "approval_id": payload.approval_id},
    )
    db.commit()

    try:
        result = _execute_known_action(action_name, payload.parameters, db=db, settings=settings)
        stdout_summary = result.pop("stdout_summary", None)
        stderr_summary = result.pop("stderr_summary", None)
        exit_code = result.pop("exit_code", None)
        status_value: Literal["completed", "failed", "blocked"] = (
            "failed" if result.get("status") == "failed" or (exit_code not in (None, 0)) else "completed"
        )
    except HTTPException:
        raise
    except Exception as exc:
        result = {"error": str(exc)}
        status_value = "failed"
        stdout_summary = None
        stderr_summary = None
        exit_code = None

    finished_at = datetime.now(UTC)
    finish_event = _audit_agent_action(
        db,
        actor_id=payload.actor_id,
        event_type="agent.action.finished",
        decision=status_value,
        action_name=action_name,
        details={
            "status": status_value,
            "started_audit_event_id": start_event.id,
            "exit_code": exit_code,
        },
    )
    db.commit()
    return AgentActionExecuteResponse(
        action_name=action_name,
        status=status_value,
        result=result,
        stdout_summary=stdout_summary,
        stderr_summary=stderr_summary,
        exit_code=exit_code,
        started_at=started_at,
        finished_at=finished_at,
        audit_event_id=finish_event.id,
    )


def _require_action_approval(
    db: Session,
    definition: AgentActionDefinition,
    payload: AgentActionExecuteRequest,
) -> None:
    if not payload.approval_id:
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Agent action requires approval")
    approval = db.get(Approval, payload.approval_id)
    if approval is None:
        raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Approval not found")
    if approval.status != "approved":
        raise HTTPException(status_code=status.HTTP_403_FORBIDDEN, detail="Approval is not approved")
    if approval.request_type != "agent_action":
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Approval is not for an agent action")
    request_payload = approval.request_payload_json or {}
    if request_payload.get("action_name") != definition.name:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Approval action_name does not match")
    approved_parameters = request_payload.get("parameters")
    if approved_parameters is not None and approved_parameters != payload.parameters:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Approval parameters do not match")


def _execute_known_action(
    action_name: str,
    parameters: dict[str, Any],
    *,
    db: Session,
    settings: Settings,
) -> dict[str, Any]:
    if action_name == "show_project_health":
        return {"health": ready()}
    if action_name == "show_git_status":
        return {"git": _git_status()}
    if action_name == "show_latest_diff":
        return {"latest_diff": _latest_diff()}
    if action_name == "show_work_items":
        return {"work_items": _work_items(db)}
    if action_name == "run_retrieval_preview":
        preview = build_retrieval_preview(
            db,
            settings,
            ChatRetrievalPreviewRequest(
                message=str(parameters["message"]),
                limit=int(parameters.get("limit", 10)),
            ),
        )
        return {"retrieval_preview": preview.model_dump(mode="json")}
    if action_name == "start_stack":
        return _run_script(["scripts/run.sh", "--detached"])
    if action_name == "stop_stack":
        return _run_script(["scripts/stop.sh"])
    if action_name == "run_last_healthy_stack":
        return _run_script(["scripts/run-last-healthy-config.sh"])
    raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Unknown agent action")


def _runtime_capabilities() -> AgentRuntimeCapabilities:
    docker_cli_available = which("docker") is not None
    docker_compose_available = False
    if docker_cli_available:
        compose_result = _run_runtime_probe(["docker", "compose", "version"])
        docker_compose_available = compose_result["exit_code"] == 0

    docker_socket_path = environ.get("DOCKER_HOST")
    docker_host_configured = bool(docker_socket_path)
    socket_path: str | None = None
    docker_socket_available = False
    if docker_socket_path and docker_socket_path.startswith("unix://"):
        socket_path = docker_socket_path.removeprefix("unix://")
        docker_socket_available = Path(socket_path).exists()
    elif docker_socket_path:
        docker_socket_available = True
    else:
        default_socket = Path("/var/run/docker.sock")
        socket_path = str(default_socket)
        docker_socket_available = default_socket.exists()

    docker_control_available = docker_cli_available and docker_compose_available and docker_socket_available
    reason = None
    if not docker_control_available:
        missing = []
        if not docker_cli_available:
            missing.append("Docker CLI is unavailable in the API runtime")
        if docker_cli_available and not docker_compose_available:
            missing.append("Docker Compose is unavailable in the API runtime")
        if not docker_socket_available:
            missing.append("Docker socket/control path is unavailable in the API runtime")
        reason = "; ".join(missing)

    return AgentRuntimeCapabilities(
        repo_root=str(repo_root()),
        docker_cli_available=docker_cli_available,
        docker_compose_available=docker_compose_available,
        docker_socket_available=docker_socket_available,
        docker_host_configured=docker_host_configured,
        docker_control_available=docker_control_available,
        docker_socket_path=socket_path,
        reason=reason,
    )


def _script_capability_reason(scripts_exist: bool, runtime: AgentRuntimeCapabilities) -> str:
    if not scripts_exist:
        return "Required DIFF-082 script is unavailable in the API runtime."
    if runtime.reason:
        return runtime.reason
    return "Stack-control action is not executable in the API runtime."


def _require_script_runtime_capability(definition: AgentActionDefinition) -> None:
    capabilities = get_agent_capabilities()
    action_capability = next((action for action in capabilities.actions if action.name == definition.name), None)
    if action_capability is None or not action_capability.executable_in_api_runtime:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={
                "message": "Agent action is not executable in the current API runtime",
                "action_name": definition.name,
                "reason": action_capability.reason if action_capability else capabilities.runtime.reason,
            },
        )


def _git_status() -> dict[str, Any]:
    root = repo_root()
    branch_result = _run_readonly_command(["git", "-C", str(root), "branch", "--show-current"])
    commit_result = _run_readonly_command(["git", "-C", str(root), "rev-parse", "HEAD"])
    status_result = _run_readonly_command(["git", "-C", str(root), "status", "--short"])
    if branch_result["exit_code"] != 0 or commit_result["exit_code"] != 0 or status_result["exit_code"] != 0:
        return _git_status_from_files(root)
    branch = branch_result["stdout"].strip()
    commit = commit_result["stdout"].strip()
    short_status = status_result["stdout"].splitlines()
    return {
        "branch": branch or "unknown",
        "commit": commit,
        "dirty": bool(short_status),
        "changed_path_count": len(short_status),
        "status_source": "git",
    }


def _git_status_from_files(root: Path) -> dict[str, Any]:
    git_dir = root / ".git"
    head_path = git_dir / "HEAD"
    if not head_path.is_file():
        return {
            "branch": "unknown",
            "commit": "unknown",
            "dirty": None,
            "changed_path_count": None,
            "status_source": "unavailable",
            "note": "Git metadata is unavailable in this runtime.",
        }
    head_value = head_path.read_text(encoding="utf-8").strip()
    branch = "detached"
    commit = head_value
    if head_value.startswith("ref: "):
        ref_name = head_value.removeprefix("ref: ").strip()
        branch = ref_name.removeprefix("refs/heads/")
        ref_path = git_dir / ref_name
        if ref_path.is_file():
            commit = ref_path.read_text(encoding="utf-8").strip()
        else:
            commit = _read_packed_ref(git_dir / "packed-refs", ref_name) or "unknown"
    return {
        "branch": branch,
        "commit": commit,
        "dirty": None,
        "changed_path_count": None,
        "status_source": "git_files",
        "note": "Git executable is unavailable; dirty state cannot be computed from raw metadata.",
    }


def _read_packed_ref(packed_refs_path: Path, ref_name: str) -> str | None:
    if not packed_refs_path.is_file():
        return None
    for line in packed_refs_path.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#") or line.startswith("^"):
            continue
        try:
            commit, packed_ref_name = line.split(" ", 1)
        except ValueError:
            continue
        if packed_ref_name == ref_name:
            return commit
    return None


def _latest_diff() -> dict[str, Any]:
    diff_dir = repo_root() / "docs" / "diffs"
    candidates = sorted(diff_dir.glob("DIFF-*.md"))
    if not candidates:
        return {"path": None, "status": None}
    latest = max(candidates, key=lambda path: path.name)
    status_line = None
    for line in latest.read_text(encoding="utf-8").splitlines():
        if line.lower().startswith("status:"):
            status_line = line.split(":", 1)[1].strip()
            break
    return {"path": str(latest.relative_to(repo_root())), "status": status_line}


def _work_items(db: Session) -> list[dict[str, Any]]:
    rows = list(db.scalars(select(WorkItem).order_by(WorkItem.created_at.desc()).limit(20)).all())
    return [
        {
            "id": item.id,
            "work_type": item.work_type,
            "status": item.status,
            "requested_by_actor_id": item.requested_by_actor_id,
            "error_message": item.error_message,
        }
        for item in rows
    ]


def _run_readonly_command(argv: list[str]) -> dict[str, Any]:
    try:
        completed = subprocess.run(
            argv,
            cwd=repo_root(),
            check=False,
            capture_output=True,
            text=True,
            timeout=10,
        )
    except FileNotFoundError:
        return {"stdout": "", "stderr": f"Command unavailable: {argv[0]}", "exit_code": 127}
    return {
        "stdout": _bounded_output(completed.stdout),
        "stderr": _bounded_output(completed.stderr),
        "exit_code": completed.returncode,
    }


def _run_runtime_probe(argv: list[str]) -> dict[str, Any]:
    try:
        completed = subprocess.run(
            argv,
            cwd=repo_root(),
            check=False,
            capture_output=True,
            text=True,
            timeout=5,
        )
    except (FileNotFoundError, subprocess.TimeoutExpired):
        return {"stdout": "", "stderr": "", "exit_code": 127}
    return {
        "stdout": _bounded_output(completed.stdout, limit=500),
        "stderr": _bounded_output(completed.stderr, limit=500),
        "exit_code": completed.returncode,
    }


def _run_script(relative_argv: list[str]) -> dict[str, Any]:
    root = repo_root()
    script_path = root / relative_argv[0]
    if not script_path.is_file():
        return {
            "status": "script_unavailable",
            "script": relative_argv[0],
            "stdout_summary": None,
            "stderr_summary": f"Script not found: {relative_argv[0]}",
            "exit_code": 127,
        }
    argv = [str(script_path), *relative_argv[1:]]
    completed = subprocess.run(
        argv,
        cwd=root,
        check=False,
        capture_output=True,
        text=True,
        timeout=300,
    )
    return {
        "script": relative_argv[0],
        "status": "completed" if completed.returncode == 0 else "failed",
        "stdout_summary": _bounded_output(completed.stdout),
        "stderr_summary": _bounded_output(completed.stderr),
        "exit_code": completed.returncode,
    }


def _bounded_output(value: str, limit: int = 4000) -> str:
    redacted_lines = []
    for line in value.splitlines():
        lowered = line.lower()
        if any(token in lowered for token in ("password", "token", "secret", "database_url", "neo4j_password")):
            redacted_lines.append("[redacted sensitive output line]")
        else:
            redacted_lines.append(line)
    redacted = "\n".join(redacted_lines)
    if len(redacted) <= limit:
        return redacted
    return redacted[: limit - 20] + "\n[output truncated]"


def _safe_parameter_summary(parameters: dict[str, Any]) -> dict[str, Any]:
    safe: dict[str, Any] = {}
    for key, value in parameters.items():
        lowered = key.lower()
        if any(token in lowered for token in ("password", "token", "secret", "key")):
            safe[key] = "[redacted]"
        else:
            safe[key] = value
    return safe


def _audit_agent_action(
    db: Session,
    *,
    actor_id: str,
    event_type: str,
    decision: str,
    action_name: str,
    details: dict[str, Any],
) -> AuditEvent:
    event = AuditEvent(
        actor_id=actor_id,
        event_type=event_type,
        decision=decision,
        resource_type="agent_action",
        resource_id=action_name,
        correlation_id=None,
        details_json=details,
    )
    db.add(event)
    db.flush()
    return event
