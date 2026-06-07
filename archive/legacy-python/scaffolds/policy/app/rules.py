from typing import Final


SOURCE_TYPES: Final[set[str]] = {
    "manual_upload",
    "local_project",
    "local_pc_diagnostics",
    "web_public",
    "web_authorized_account",
    "router_network",
    "user_observation",
    "conversation_history",
}

ALLOWED_OPERATIONS: Final[set[str]] = {
    "dry_run",
    "read",
    "collect",
    "normalize",
    "classify_sensitivity",
    "extract_metadata",
}

SENSITIVITY_LABELS: Final[set[str]] = {
    "public",
    "internal",
    "sensitive",
    "secret",
}

EXTERNAL_MODEL_POLICIES: Final[set[str]] = {
    "blocked",
    "metadata_only",
    "allowed_with_approval",
}

SENSITIVE_OPERATIONS: Final[set[str]] = {
    "collect",
    "external_model_use",
    "file_write",
    "account_action",
    "router_change",
    "repository_write",
    "website_change",
    "sensitive_export",
}


def is_known_source_type(source_type: str) -> bool:
    return source_type in SOURCE_TYPES


def is_known_sensitivity(sensitivity: str) -> bool:
    return sensitivity in SENSITIVITY_LABELS


def is_known_external_model_policy(policy: str) -> bool:
    return policy in EXTERNAL_MODEL_POLICIES


def unknown_operations(operations: list[str]) -> list[str]:
    return [operation for operation in operations if operation not in ALLOWED_OPERATIONS]


def requires_approval(
    *,
    operation: str,
    sensitivity: str,
    external_model_policy: str = "blocked",
) -> bool:
    if operation in SENSITIVE_OPERATIONS:
        return True
    if sensitivity in {"sensitive", "secret"}:
        return True
    return external_model_policy == "allowed_with_approval"
