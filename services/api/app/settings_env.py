import hashlib
import os
import shutil
import subprocess
import tempfile
from dataclasses import dataclass
from datetime import UTC, datetime
from pathlib import Path
from typing import Any
from urllib.parse import urlparse

from fastapi import APIRouter, Depends, HTTPException, status
from pydantic import BaseModel, Field
from sqlalchemy.orm import Session

from app.config import Settings, get_settings
from app.db import get_db
from app.models import AuditEvent

router = APIRouter(prefix="/settings/env", tags=["settings"])

MASKED_VALUE = "********"
READ_ONLY_KEYS = {"ENV_FILE_PATH", "ENV_BACKUP_DIR"}
DEFAULT_IGY6_DATA_ROOT = "../IGY6_Data"
SECRET_KEYS = {"POSTGRES_PASSWORD", "DATABASE_URL", "NEO4J_PASSWORD"}
BOOLEAN_KEYS = {"SINGLE_USER_MODE", "APPROVAL_REQUIRED_DEFAULT"}
PORT_KEYS = {
    "APP_PORT",
    "POSTGRES_PORT",
    "REDIS_PORT",
    "QDRANT_PORT",
    "NEO4J_HTTP_PORT",
    "NEO4J_BOLT_PORT",
    "PHOENIX_PORT",
}
URL_KEYS = {
    "API_BASE_URL",
    "WEB_BASE_URL",
    "DATABASE_URL",
    "REDIS_URL",
    "CELERY_BROKER_URL",
    "CELERY_RESULT_BACKEND",
    "QDRANT_URL",
    "NEO4J_URI",
    "MLFLOW_TRACKING_URI",
    "PHOENIX_COLLECTOR_ENDPOINT",
}
STORAGE_KEYS = {"ARTIFACT_STORE_PATH", "EXPORT_STORE_PATH", "ENV_FILE_PATH", "ENV_BACKUP_DIR"}
HOST_PATH_KEYS = {"IGY6_DATA_ROOT"}
EXTERNAL_MODEL_POLICIES = {"blocked", "metadata_only", "allowed_with_approval"}
AUDIT_LOG_LEVELS = {"debug", "info", "warning", "error"}
RESTART_KEYS = {
    "APP_ENV",
    "APP_HOST",
    "APP_PORT",
    "API_BASE_URL",
    "WEB_BASE_URL",
    "POSTGRES_HOST",
    "POSTGRES_PORT",
    "POSTGRES_DB",
    "POSTGRES_USER",
    "POSTGRES_PASSWORD",
    "DATABASE_URL",
    "REDIS_HOST",
    "REDIS_PORT",
    "REDIS_URL",
    "CELERY_BROKER_URL",
    "CELERY_RESULT_BACKEND",
    "QDRANT_HOST",
    "QDRANT_PORT",
    "QDRANT_URL",
    "QDRANT_CHUNK_COLLECTION",
    "QDRANT_CHUNK_VECTOR_SIZE",
    "NEO4J_HOST",
    "NEO4J_HTTP_PORT",
    "NEO4J_BOLT_PORT",
    "NEO4J_USER",
    "NEO4J_PASSWORD",
    "NEO4J_URI",
    "MLFLOW_TRACKING_URI",
    "MLFLOW_ARTIFACT_ROOT",
    "PHOENIX_HOST",
    "PHOENIX_PORT",
    "PHOENIX_COLLECTOR_ENDPOINT",
    "ARTIFACT_STORE_PATH",
    "EXPORT_STORE_PATH",
    "EXTERNAL_MODEL_POLICY_DEFAULT",
    "SINGLE_USER_MODE",
    "AUDIT_LOG_LEVEL",
    "APPROVAL_REQUIRED_DEFAULT",
    "ENV_FILE_PATH",
    "ENV_BACKUP_DIR",
    "IGY6_DATA_ROOT",
}
SAFE_ENV_FILE_PATH = Path("/workspace/project/.env")
SAFE_BACKUP_ROOT = Path("/workspace/storage")

SETTING_GROUPS = [
    ("app", "App / Web"),
    ("postgres", "PostgreSQL"),
    ("redis", "Redis / Celery"),
    ("qdrant", "Qdrant"),
    ("neo4j", "Neo4j"),
    ("mlflow", "MLflow"),
    ("phoenix", "Phoenix"),
    ("storage", "Storage"),
    ("policy", "Policy / Safety"),
]

SETTING_DEFINITIONS: list[dict[str, str]] = [
    {"key": "APP_ENV", "group": "app", "description": "Local application environment label."},
    {"key": "APP_HOST", "group": "app", "description": "API bind host used by local configuration."},
    {"key": "APP_PORT", "group": "app", "description": "Published local API port."},
    {"key": "API_BASE_URL", "group": "app", "description": "Browser-facing API base URL."},
    {"key": "WEB_BASE_URL", "group": "app", "description": "Browser-facing web UI base URL."},
    {"key": "POSTGRES_HOST", "group": "postgres", "description": "PostgreSQL service hostname."},
    {"key": "POSTGRES_PORT", "group": "postgres", "description": "Published local PostgreSQL port."},
    {"key": "POSTGRES_DB", "group": "postgres", "description": "PostgreSQL database name."},
    {"key": "POSTGRES_USER", "group": "postgres", "description": "PostgreSQL username."},
    {"key": "POSTGRES_PASSWORD", "group": "postgres", "description": "PostgreSQL password."},
    {"key": "DATABASE_URL", "group": "postgres", "description": "SQLAlchemy PostgreSQL connection URL."},
    {"key": "REDIS_HOST", "group": "redis", "description": "Redis service hostname."},
    {"key": "REDIS_PORT", "group": "redis", "description": "Published local Redis port."},
    {"key": "REDIS_URL", "group": "redis", "description": "Redis URL used by API health checks."},
    {"key": "CELERY_BROKER_URL", "group": "redis", "description": "Celery broker Redis URL."},
    {"key": "CELERY_RESULT_BACKEND", "group": "redis", "description": "Celery result backend Redis URL."},
    {"key": "QDRANT_HOST", "group": "qdrant", "description": "Qdrant service hostname."},
    {"key": "QDRANT_PORT", "group": "qdrant", "description": "Published local Qdrant port."},
    {"key": "QDRANT_URL", "group": "qdrant", "description": "Qdrant API URL used by API and worker."},
    {"key": "QDRANT_CHUNK_COLLECTION", "group": "qdrant", "description": "Qdrant collection for chunk vectors."},
    {"key": "QDRANT_CHUNK_VECTOR_SIZE", "group": "qdrant", "description": "Deterministic chunk vector size."},
    {"key": "NEO4J_HOST", "group": "neo4j", "description": "Neo4j service hostname."},
    {"key": "NEO4J_HTTP_PORT", "group": "neo4j", "description": "Published local Neo4j browser port."},
    {"key": "NEO4J_BOLT_PORT", "group": "neo4j", "description": "Published local Neo4j Bolt port."},
    {"key": "NEO4J_USER", "group": "neo4j", "description": "Neo4j username."},
    {"key": "NEO4J_PASSWORD", "group": "neo4j", "description": "Neo4j password."},
    {"key": "NEO4J_URI", "group": "neo4j", "description": "Neo4j Bolt URI used by the API."},
    {"key": "MLFLOW_TRACKING_URI", "group": "mlflow", "description": "Reserved local MLflow tracking URI."},
    {"key": "MLFLOW_ARTIFACT_ROOT", "group": "mlflow", "description": "Reserved MLflow artifact root inside the service."},
    {"key": "PHOENIX_HOST", "group": "phoenix", "description": "Phoenix service hostname."},
    {"key": "PHOENIX_PORT", "group": "phoenix", "description": "Published local Phoenix port."},
    {"key": "PHOENIX_COLLECTOR_ENDPOINT", "group": "phoenix", "description": "Reserved local Phoenix endpoint."},
    {"key": "ARTIFACT_STORE_PATH", "group": "storage", "description": "Container path for content-addressed artifacts."},
    {"key": "EXPORT_STORE_PATH", "group": "storage", "description": "Container path for report/export output."},
    {"key": "ENV_FILE_PATH", "group": "storage", "description": "Controlled container path to the mounted local .env file."},
    {"key": "ENV_BACKUP_DIR", "group": "storage", "description": "Controlled backup directory for .env backups."},
    {
        "key": "IGY6_DATA_ROOT",
        "group": "storage",
        "description": (
            "Host-side folder where IGY6 stores database, vector, graph, artifact, "
            "report, backup, MLflow, and Phoenix runtime data."
        ),
    },
    {"key": "EXTERNAL_MODEL_POLICY_DEFAULT", "group": "policy", "description": "Default external model policy."},
    {"key": "SINGLE_USER_MODE", "group": "policy", "description": "Local single-user mode toggle."},
    {"key": "AUDIT_LOG_LEVEL", "group": "policy", "description": "Audit logging verbosity label."},
    {"key": "APPROVAL_REQUIRED_DEFAULT", "group": "policy", "description": "Default approval-required toggle."},
]

SETTING_BY_KEY = {definition["key"]: definition for definition in SETTING_DEFINITIONS}
ALLOWLIST = set(SETTING_BY_KEY)


class EnvSettingRead(BaseModel):
    key: str
    group: str
    group_label: str
    description: str
    value: str | None = None
    masked_value: str | None = None
    has_value: bool
    secret: bool
    read_only: bool
    restart_required: bool
    source: str


class EnvUnmanagedRead(BaseModel):
    key: str
    masked_value: str
    has_value: bool
    secret: bool
    read_only: bool = True


class EnvFileStatus(BaseModel):
    path: str
    backup_dir: str
    exists: bool
    writable: bool
    unknown_key_count: int
    output_format: str


class EnvSettingsResponse(BaseModel):
    file_status: EnvFileStatus
    groups: list[dict[str, str]]
    settings: list[EnvSettingRead]
    unmanaged: list[EnvUnmanagedRead]
    warnings: list[str]


class EnvCandidateRequest(BaseModel):
    values: dict[str, str] = Field(default_factory=dict)
    actor_id: str = "local-owner"


class EnvApplyRequest(EnvCandidateRequest):
    verification_token: str = Field(min_length=1)


class EnvValidationIssue(BaseModel):
    key: str | None = None
    message: str


class EnvVerifyResponse(BaseModel):
    passed: bool
    errors: list[EnvValidationIssue]
    warnings: list[EnvValidationIssue]
    normalized_candidate: list[EnvSettingRead]
    changed_keys: list[str]
    restart_required: bool
    restart_notes: list[str]
    verification_token: str | None = None
    candidate_hash: str | None = None
    expires_at: str | None = None
    compose_validation: dict[str, Any]


class EnvApplyResponse(BaseModel):
    saved: bool
    backup_path: str
    changed_keys: list[str]
    restart_required: bool
    restart_notes: list[str]
    warnings: list[EnvValidationIssue]
    current: EnvSettingsResponse


@dataclass(frozen=True)
class ParsedEnv:
    values: dict[str, str]
    order: list[str]
    unmanaged_order: list[str]


@dataclass(frozen=True)
class ValidationResult:
    passed: bool
    errors: list[EnvValidationIssue]
    warnings: list[EnvValidationIssue]
    changed_keys: list[str]
    restart_required: bool
    restart_notes: list[str]
    candidate_hash: str
    compose_validation: dict[str, Any]


def is_secret_key(key: str) -> bool:
    if key in SECRET_KEYS:
        return True
    upper = key.upper()
    if any(token in upper for token in ("PASSWORD", "TOKEN", "SECRET")):
        return True
    return "KEY" in upper and upper not in {"QDRANT_CHUNK_COLLECTION"}


def parse_env_content(content: str) -> ParsedEnv:
    values: dict[str, str] = {}
    order: list[str] = []
    unmanaged_order: list[str] = []
    for raw_line in content.splitlines():
        stripped = raw_line.strip()
        if not stripped or stripped.startswith("#") or "=" not in raw_line:
            continue
        key, value = raw_line.split("=", 1)
        key = key.strip()
        if not key:
            continue
        parsed_value = value.strip()
        if (
            len(parsed_value) >= 2
            and parsed_value[0] == parsed_value[-1]
            and parsed_value[0] in {"'", '"'}
        ):
            parsed_value = parsed_value[1:-1]
        values[key] = parsed_value
        if key in ALLOWLIST:
            if key not in order:
                order.append(key)
        elif key not in unmanaged_order:
            unmanaged_order.append(key)
    return ParsedEnv(values=values, order=order, unmanaged_order=unmanaged_order)


def render_env_content(values: dict[str, str], unmanaged: dict[str, str] | None = None) -> str:
    group_lookup = {group_key: group_label for group_key, group_label in SETTING_GROUPS}
    lines = [
        "# IGY6 local environment",
        "# Generated by the local Settings dry-run/apply workflow.",
        "# Comments from previous .env content are normalized by this writer.",
        "",
    ]
    current_group: str | None = None
    for definition in SETTING_DEFINITIONS:
        key = definition["key"]
        group = definition["group"]
        if current_group != group:
            if current_group is not None:
                lines.append("")
            lines.append(f"# {group_lookup[group]}")
            current_group = group
        lines.append(f"{key}={values.get(key, '')}")

    preserved = unmanaged or {}
    if preserved:
        lines.extend(["", "# Unmanaged keys preserved read-only"])
        for key in sorted(preserved):
            lines.append(f"{key}={preserved[key]}")
    lines.append("")
    return "\n".join(lines)


def candidate_hash(values: dict[str, str], unmanaged: dict[str, str] | None = None) -> str:
    content = render_env_content(values, unmanaged)
    return hashlib.sha256(content.encode("utf-8")).hexdigest()


def _settings_paths(settings: Settings) -> tuple[Path, Path]:
    return Path(settings.env_file_path).expanduser(), Path(settings.env_backup_dir).expanduser()


def _is_configured_env_path_safe(env_path: Path, backup_dir: Path) -> bool:
    resolved_env = env_path.resolve(strict=False)
    resolved_backup = backup_dir.resolve(strict=False)
    try:
        resolved_backup.relative_to(SAFE_BACKUP_ROOT)
    except ValueError:
        return False
    return resolved_env == SAFE_ENV_FILE_PATH


def _read_current_env(settings: Settings) -> ParsedEnv:
    env_path, _ = _settings_paths(settings)
    if not env_path.exists():
        return ParsedEnv(values={}, order=[], unmanaged_order=[])
    return parse_env_content(env_path.read_text(encoding="utf-8"))


def _base_values(parsed: ParsedEnv, settings: Settings) -> dict[str, str]:
    env_path, backup_dir = _settings_paths(settings)
    values = {key: parsed.values[key] for key in parsed.values if key in ALLOWLIST}
    defaults = {
        "ENV_FILE_PATH": str(env_path),
        "ENV_BACKUP_DIR": str(backup_dir),
        "IGY6_DATA_ROOT": settings.igy6_data_root,
    }
    for key, value in defaults.items():
        values.setdefault(key, value)
    return values


def _unmanaged_values(parsed: ParsedEnv) -> dict[str, str]:
    return {key: parsed.values[key] for key in parsed.unmanaged_order if key not in ALLOWLIST}


def _group_label(group: str) -> str:
    return dict(SETTING_GROUPS).get(group, group)


def _sanitize_setting(key: str, value: str | None, source: str = "env") -> EnvSettingRead:
    definition = SETTING_BY_KEY[key]
    secret = is_secret_key(key)
    has_value = value is not None and value != ""
    return EnvSettingRead(
        key=key,
        group=definition["group"],
        group_label=_group_label(definition["group"]),
        description=definition["description"],
        value=None if secret else value,
        masked_value=MASKED_VALUE if secret and has_value else "",
        has_value=has_value,
        secret=secret,
        read_only=key in READ_ONLY_KEYS,
        restart_required=key in RESTART_KEYS,
        source=source,
    )


def sanitize_settings(values: dict[str, str], source: str = "env") -> list[EnvSettingRead]:
    return [_sanitize_setting(definition["key"], values.get(definition["key"]), source) for definition in SETTING_DEFINITIONS]


def _file_status(settings: Settings, parsed: ParsedEnv) -> EnvFileStatus:
    env_path, backup_dir = _settings_paths(settings)
    return EnvFileStatus(
        path=str(env_path),
        backup_dir=str(backup_dir),
        exists=env_path.exists(),
        writable=env_path.exists() and os.access(env_path, os.W_OK),
        unknown_key_count=len(parsed.unmanaged_order),
        output_format="normalized_env",
    )


def _parse_bool(value: str) -> bool:
    normalized = value.strip().lower()
    if normalized in {"true", "1", "yes", "on"}:
        return True
    if normalized in {"false", "0", "no", "off"}:
        return False
    raise ValueError("expected true or false")


def _is_plausible_url(key: str, value: str) -> bool:
    parsed = urlparse(value)
    if key == "NEO4J_URI":
        return parsed.scheme in {"bolt", "neo4j"} and bool(parsed.hostname)
    if key == "DATABASE_URL":
        return parsed.scheme.startswith("postgresql") and bool(parsed.hostname) and bool(parsed.path.strip("/"))
    if key in {"REDIS_URL", "CELERY_BROKER_URL", "CELERY_RESULT_BACKEND"}:
        return parsed.scheme == "redis" and bool(parsed.hostname)
    return parsed.scheme in {"http", "https"} and bool(parsed.hostname)


def _validate_agreement(
    *,
    values: dict[str, str],
    url_key: str,
    host_key: str,
    port_key: str,
    errors: list[EnvValidationIssue],
    user_key: str | None = None,
    password_key: str | None = None,
    database_key: str | None = None,
) -> None:
    parsed = urlparse(values.get(url_key, ""))
    if parsed.hostname and parsed.hostname != values.get(host_key):
        errors.append(EnvValidationIssue(key=url_key, message=f"{url_key} host must match {host_key}."))
    if parsed.port and str(parsed.port) != values.get(port_key):
        errors.append(EnvValidationIssue(key=url_key, message=f"{url_key} port must match {port_key}."))
    if user_key and parsed.username and parsed.username != values.get(user_key):
        errors.append(EnvValidationIssue(key=url_key, message=f"{url_key} username must match {user_key}."))
    if password_key and parsed.password and parsed.password != values.get(password_key):
        errors.append(EnvValidationIssue(key=url_key, message=f"{url_key} password must match {password_key}."))
    if database_key and parsed.path.strip("/") != values.get(database_key):
        errors.append(EnvValidationIssue(key=url_key, message=f"{url_key} database name must match {database_key}."))


def _storage_path_is_safe(value: str) -> bool:
    path = Path(value)
    return path.is_absolute() and ".." not in path.parts


def _host_data_root_issue(value: str) -> str | None:
    stripped = value.strip()
    if not stripped:
        return "IGY6_DATA_ROOT must not be empty."
    normalized = stripped.replace("\\", "/")
    if normalized == DEFAULT_IGY6_DATA_ROOT:
        return None
    if normalized in {"/", "~"}:
        return "IGY6_DATA_ROOT must point to a dedicated folder, not a filesystem root."
    windows_drive_root = len(normalized) == 3 and normalized[1:] == ":/" and normalized[0].isalpha()
    if windows_drive_root:
        return "IGY6_DATA_ROOT must not be a drive root such as C:/ or D:/."
    if "\\" in stripped:
        return "Use forward slashes in IGY6_DATA_ROOT, for example D:/Projects/IGY6_Data."
    windows_absolute = len(normalized) > 3 and normalized[1:3] == ":/" and normalized[0].isalpha()
    linux_absolute = normalized.startswith("/")
    if not windows_absolute and not linux_absolute and normalized != DEFAULT_IGY6_DATA_ROOT:
        return "Use ../IGY6_Data or an absolute path such as D:/Projects/IGY6_Data or /home/user/IGY6_Data."
    parts = [part for part in normalized.split("/") if part]
    if ".." in parts:
        return "IGY6_DATA_ROOT must not contain path traversal, except for the default ../IGY6_Data."
    return None


def _compose_validate(candidate: dict[str, str], unmanaged: dict[str, str]) -> dict[str, Any]:
    docker_path = shutil.which("docker")
    compose_path = Path("/workspace/project/infra/docker-compose.yml")
    if docker_path is None:
        return {
            "available": False,
            "passed": None,
            "message": "Docker CLI is unavailable from this runtime; Compose validation was not run.",
        }
    if not compose_path.exists():
        return {
            "available": False,
            "passed": None,
            "message": "Compose file is unavailable from this runtime; Compose validation was not run.",
        }
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as handle:
        handle.write(render_env_content(candidate, unmanaged))
        temp_path = handle.name
    try:
        result = subprocess.run(
            [
                docker_path,
                "compose",
                "-f",
                str(compose_path),
                "--env-file",
                temp_path,
                "config",
            ],
            check=False,
            capture_output=True,
            text=True,
            timeout=15,
        )
    except Exception as exc:
        return {"available": True, "passed": False, "message": str(exc)}
    finally:
        Path(temp_path).unlink(missing_ok=True)
    return {
        "available": True,
        "passed": result.returncode == 0,
        "message": "Compose config validation passed." if result.returncode == 0 else result.stderr[-1200:],
    }


def build_candidate(
    *,
    settings: Settings,
    requested_values: dict[str, str],
    parsed: ParsedEnv | None = None,
) -> tuple[dict[str, str], dict[str, str], list[str]]:
    parsed_env = parsed or _read_current_env(settings)
    base = _base_values(parsed_env, settings)
    unmanaged = _unmanaged_values(parsed_env)
    unknown_changes = sorted(key for key in requested_values if key not in ALLOWLIST)
    read_only_changes = sorted(key for key in requested_values if key in READ_ONLY_KEYS)
    if unknown_changes:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail={"message": "Unknown settings keys are read-only unmanaged keys.", "keys": unknown_changes},
        )
    if read_only_changes:
        raise HTTPException(
            status_code=status.HTTP_422_UNPROCESSABLE_ENTITY,
            detail={"message": "Read-only settings cannot be changed.", "keys": read_only_changes},
        )
    candidate = {key: base.get(key, "") for key in ALLOWLIST}
    for key, value in requested_values.items():
        candidate[key] = value
    changed_keys = sorted(key for key, value in candidate.items() if value != base.get(key, ""))
    return candidate, unmanaged, changed_keys


def validate_candidate(candidate: dict[str, str], unmanaged: dict[str, str], changed_keys: list[str]) -> ValidationResult:
    errors: list[EnvValidationIssue] = []
    warnings: list[EnvValidationIssue] = []
    for key in sorted(ALLOWLIST - READ_ONLY_KEYS):
        if key not in candidate or candidate[key] == "":
            errors.append(EnvValidationIssue(key=key, message="Required setting is missing."))

    for key in PORT_KEYS:
        try:
            value = int(candidate.get(key, ""))
        except ValueError:
            errors.append(EnvValidationIssue(key=key, message="Port must be an integer."))
            continue
        if value < 1 or value > 65535:
            errors.append(EnvValidationIssue(key=key, message="Port must be between 1 and 65535."))

    for key in BOOLEAN_KEYS:
        try:
            _parse_bool(candidate.get(key, ""))
        except ValueError:
            errors.append(EnvValidationIssue(key=key, message="Boolean must be true or false."))

    for key in URL_KEYS:
        value = candidate.get(key, "")
        if value and not _is_plausible_url(key, value):
            errors.append(EnvValidationIssue(key=key, message="URL or URI is not syntactically plausible."))

    if candidate.get("DATABASE_URL"):
        _validate_agreement(
            values=candidate,
            url_key="DATABASE_URL",
            host_key="POSTGRES_HOST",
            port_key="POSTGRES_PORT",
            user_key="POSTGRES_USER",
            password_key="POSTGRES_PASSWORD",
            database_key="POSTGRES_DB",
            errors=errors,
        )
    if candidate.get("NEO4J_URI"):
        _validate_agreement(
            values=candidate,
            url_key="NEO4J_URI",
            host_key="NEO4J_HOST",
            port_key="NEO4J_BOLT_PORT",
            errors=errors,
        )
    if candidate.get("QDRANT_URL"):
        _validate_agreement(
            values=candidate,
            url_key="QDRANT_URL",
            host_key="QDRANT_HOST",
            port_key="QDRANT_PORT",
            errors=errors,
        )

    for key in STORAGE_KEYS:
        value = candidate.get(key, "")
        if value and not _storage_path_is_safe(value):
            errors.append(EnvValidationIssue(key=key, message="Storage path must be absolute and must not contain traversal."))

    for key in HOST_PATH_KEYS:
        issue = _host_data_root_issue(candidate.get(key, ""))
        if issue is not None:
            errors.append(EnvValidationIssue(key=key, message=issue))

    if not _is_configured_env_path_safe(Path(candidate.get("ENV_FILE_PATH", "")), Path(candidate.get("ENV_BACKUP_DIR", ""))):
        errors.append(
            EnvValidationIssue(
                key="ENV_FILE_PATH",
                message="Settings editor can only target /workspace/project/.env with backups under /workspace/storage.",
            )
        )

    if candidate.get("EXTERNAL_MODEL_POLICY_DEFAULT") not in EXTERNAL_MODEL_POLICIES:
        errors.append(
            EnvValidationIssue(
                key="EXTERNAL_MODEL_POLICY_DEFAULT",
                message="External model policy must be blocked, metadata_only, or allowed_with_approval.",
            )
        )
    if candidate.get("AUDIT_LOG_LEVEL", "").lower() not in AUDIT_LOG_LEVELS:
        errors.append(EnvValidationIssue(key="AUDIT_LOG_LEVEL", message="Audit log level must be debug, info, warning, or error."))

    try:
        vector_size = int(candidate.get("QDRANT_CHUNK_VECTOR_SIZE", ""))
        if vector_size < 1:
            errors.append(EnvValidationIssue(key="QDRANT_CHUNK_VECTOR_SIZE", message="Vector size must be positive."))
        elif "QDRANT_CHUNK_VECTOR_SIZE" in changed_keys:
            warnings.append(
                EnvValidationIssue(
                    key="QDRANT_CHUNK_VECTOR_SIZE",
                    message="Changing vector size can require rebuilding vector storage.",
                )
            )
    except ValueError:
        errors.append(EnvValidationIssue(key="QDRANT_CHUNK_VECTOR_SIZE", message="Vector size must be a positive integer."))

    restart_changed = sorted(key for key in changed_keys if key in RESTART_KEYS)
    restart_required = bool(restart_changed)
    restart_notes = [
        "Saved settings are written to .env only; running containers do not receive them until restart or recreate.",
    ]
    if restart_required:
        restart_notes.append(
            "Changed keys likely requiring Docker stack restart/recreate: " + ", ".join(restart_changed)
        )
    if "IGY6_DATA_ROOT" in changed_keys:
        warnings.append(
            EnvValidationIssue(
                key="IGY6_DATA_ROOT",
                message="Changing IGY6_DATA_ROOT requires Docker stack restart/recreate and does not migrate existing data.",
            )
        )
        warnings.append(
            EnvValidationIssue(
                key="IGY6_DATA_ROOT",
                message="The target data folder must already exist or be creatable by Docker.",
            )
        )

    if restart_required:
        for key in restart_changed:
            if key in {"DATABASE_URL", "POSTGRES_HOST", "POSTGRES_PORT", "POSTGRES_DB", "POSTGRES_USER", "POSTGRES_PASSWORD"}:
                warnings.append(EnvValidationIssue(key=key, message="Database changes may require stack recreate and migration checks."))
            elif key.startswith("REDIS") or key.startswith("CELERY"):
                warnings.append(EnvValidationIssue(key=key, message="Redis/Celery changes may require API, worker, and beat restart."))
            elif key.startswith("QDRANT"):
                warnings.append(EnvValidationIssue(key=key, message="Qdrant changes may require vector collection review."))
            elif key.startswith("NEO4J"):
                warnings.append(EnvValidationIssue(key=key, message="Neo4j changes may require graph connectivity review."))
            elif key.startswith("MLFLOW") or key.startswith("PHOENIX"):
                warnings.append(EnvValidationIssue(key=key, message="Reserved service changes may require stack restart."))
            elif key in STORAGE_KEYS:
                warnings.append(EnvValidationIssue(key=key, message="Storage path changes may require mounted volume review."))
            elif key in HOST_PATH_KEYS:
                warnings.append(EnvValidationIssue(key=key, message="Host data-root changes may require moving data manually while the stack is stopped."))

    compose_validation = _compose_validate(candidate, unmanaged)
    if compose_validation.get("passed") is False:
        errors.append(EnvValidationIssue(key=None, message=f"Compose validation failed: {compose_validation.get('message')}"))
    elif compose_validation.get("available") is False:
        warnings.append(EnvValidationIssue(key=None, message=compose_validation["message"]))

    hash_value = candidate_hash(candidate, unmanaged)
    return ValidationResult(
        passed=not errors,
        errors=errors,
        warnings=warnings,
        changed_keys=changed_keys,
        restart_required=restart_required,
        restart_notes=restart_notes,
        candidate_hash=hash_value,
        compose_validation=compose_validation,
    )


def _verify_response(candidate: dict[str, str], validation: ValidationResult) -> EnvVerifyResponse:
    token = validation.candidate_hash if validation.passed else None
    return EnvVerifyResponse(
        passed=validation.passed,
        errors=validation.errors,
        warnings=validation.warnings,
        normalized_candidate=sanitize_settings(candidate, source="candidate"),
        changed_keys=validation.changed_keys,
        restart_required=validation.restart_required,
        restart_notes=validation.restart_notes,
        verification_token=token,
        candidate_hash=token,
        expires_at=None,
        compose_validation=validation.compose_validation,
    )


def build_env_settings_response(settings: Settings) -> EnvSettingsResponse:
    parsed = _read_current_env(settings)
    values = _base_values(parsed, settings)
    unmanaged = _unmanaged_values(parsed)
    warnings = []
    env_path, _ = _settings_paths(settings)
    if not env_path.exists():
        warnings.append("Configured .env file does not exist yet.")
    if parsed.unmanaged_order:
        warnings.append("Unknown .env keys are preserved as read-only unmanaged settings.")
    return EnvSettingsResponse(
        file_status=_file_status(settings, parsed),
        groups=[{"key": key, "label": label} for key, label in SETTING_GROUPS],
        settings=sanitize_settings(values),
        unmanaged=[
            EnvUnmanagedRead(
                key=key,
                masked_value=MASKED_VALUE if is_secret_key(key) and unmanaged[key] else unmanaged[key],
                has_value=bool(unmanaged[key]),
                secret=is_secret_key(key),
            )
            for key in parsed.unmanaged_order
        ],
        warnings=warnings,
    )


def _create_backup(env_path: Path, backup_dir: Path) -> Path:
    if not env_path.exists():
        raise RuntimeError("Cannot back up missing .env file")
    backup_dir.mkdir(parents=True, exist_ok=True)
    timestamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%SZ")
    backup_path = backup_dir / f".env.{timestamp}.bak"
    counter = 1
    while backup_path.exists():
        backup_path = backup_dir / f".env.{timestamp}.{counter}.bak"
        counter += 1
    shutil.copy2(env_path, backup_path)
    return backup_path


def _atomic_write_env(env_path: Path, content: str) -> None:
    env_path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", encoding="utf-8", dir=str(env_path.parent), delete=False) as handle:
        handle.write(content)
        temp_path = Path(handle.name)
    os.replace(temp_path, env_path)


def _audit_env_update(
    db: Session,
    *,
    actor_id: str,
    changed_keys: list[str],
    backup_path: Path,
    validation: ValidationResult,
) -> None:
    db.add(
        AuditEvent(
            actor_id=actor_id,
            event_type="settings.env.updated",
            decision="saved",
            resource_type="settings_env",
            resource_id="local-env",
            correlation_id=validation.candidate_hash,
            details_json={
                "changed_keys": changed_keys,
                "backup_path": str(backup_path),
                "restart_required": validation.restart_required,
                "warning_count": len(validation.warnings),
                "error_count": len(validation.errors),
                "candidate_hash": validation.candidate_hash,
                "secret_values_recorded": False,
            },
        )
    )


@router.get("", response_model=EnvSettingsResponse)
def get_env_settings(settings: Settings = Depends(get_settings)) -> EnvSettingsResponse:
    return build_env_settings_response(settings)


@router.post("/verify", response_model=EnvVerifyResponse)
def verify_env_settings(
    payload: EnvCandidateRequest,
    settings: Settings = Depends(get_settings),
) -> EnvVerifyResponse:
    parsed = _read_current_env(settings)
    candidate, unmanaged, changed_keys = build_candidate(
        settings=settings,
        requested_values=payload.values,
        parsed=parsed,
    )
    validation = validate_candidate(candidate, unmanaged, changed_keys)
    return _verify_response(candidate, validation)


@router.post("/apply", response_model=EnvApplyResponse)
def apply_env_settings(
    payload: EnvApplyRequest,
    db: Session = Depends(get_db),
    settings: Settings = Depends(get_settings),
) -> EnvApplyResponse:
    parsed = _read_current_env(settings)
    candidate, unmanaged, changed_keys = build_candidate(
        settings=settings,
        requested_values=payload.values,
        parsed=parsed,
    )
    validation = validate_candidate(candidate, unmanaged, changed_keys)
    if not validation.passed:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail={"message": "Verified candidate no longer passes validation.", "errors": [issue.model_dump() for issue in validation.errors]},
        )
    if payload.verification_token != validation.candidate_hash:
        raise HTTPException(
            status_code=status.HTTP_409_CONFLICT,
            detail="Submitted settings do not match the passing dry-run verification token.",
        )

    env_path, backup_dir = _settings_paths(settings)
    if not _is_configured_env_path_safe(env_path, backup_dir):
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail="Configured .env path is not safe.")

    try:
        backup_path = _create_backup(env_path, backup_dir)
        _atomic_write_env(env_path, render_env_content(candidate, unmanaged))
    except Exception as exc:
        raise HTTPException(status_code=status.HTTP_409_CONFLICT, detail=str(exc)) from exc

    _audit_env_update(
        db,
        actor_id=payload.actor_id,
        changed_keys=changed_keys,
        backup_path=backup_path,
        validation=validation,
    )
    db.commit()
    return EnvApplyResponse(
        saved=True,
        backup_path=str(backup_path),
        changed_keys=changed_keys,
        restart_required=validation.restart_required,
        restart_notes=validation.restart_notes,
        warnings=validation.warnings,
        current=build_env_settings_response(settings),
    )
