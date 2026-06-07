#!/usr/bin/env bash
set -Eeuo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPO_ROOT="${REPO_ROOT}" python3 - "$@" <<'PY'
from __future__ import annotations

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import urllib.error
import urllib.request
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


REPO_ROOT = Path(os.environ["REPO_ROOT"]).resolve()
DEFAULT_OUTPUT_DIR = REPO_ROOT / ".igy6-local" / "diagnostics"
SMOKE_RESULT_DIR = REPO_ROOT / ".igy6-local" / "smoke-results"
SCHEMA_VERSION = "igy6.diagnostics_bundle.v1"
ROUTE_CHECKS: tuple[tuple[str, str], ...] = (
    ("api_live", "/health/live"),
    ("api_ready", "/health/ready"),
    ("rust_migration_status", "/rust-migration/status"),
    ("vector_chunks", "/memory/vector/chunks"),
    ("graph_schema", "/memory/graph/schema"),
)
TOOLS = ("git", "node", "npm", "cargo", "rustc", "python3", "docker")
SENSITIVE_RE = re.compile(r"(secret|token|password|cookie|credential|private[_-]?key|api[_-]?key)", re.IGNORECASE)
PRIVATE_PATH_RE = re.compile(r"(^|[\s\"'=])((/home|/Users|/mnt|/var|/tmp)/[A-Za-z0-9_.@+/-]+|[A-Za-z]:\\[^\\]+)")


def utc_now() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def run_command(args: list[str]) -> tuple[int, str]:
    try:
        result = subprocess.run(
            args,
            cwd=REPO_ROOT,
            text=True,
            capture_output=True,
            check=False,
            timeout=8,
        )
    except (OSError, subprocess.TimeoutExpired) as exc:
        return 1, str(exc)
    output = (result.stdout or result.stderr).strip()
    return result.returncode, output[:800]


def git_value(*args: str) -> str | None:
    code, output = run_command(["git", *args])
    if code != 0:
        return None
    return output or None


def safe_git_status() -> dict[str, Any]:
    code, output = run_command(["git", "status", "--short"])
    lines = [line for line in output.splitlines() if line.strip()] if output else []
    return {
        "command_status": "ok" if code == 0 else "failed",
        "dirty": bool(lines),
        "changed_entry_count": len(lines),
        "changed_entry_prefixes": sorted({line[:2].strip() or "unknown" for line in lines})[:12],
    }


def read_json_route(api_base_url: str, path: str, timeout: float) -> dict[str, Any]:
    request = urllib.request.Request(f"{api_base_url.rstrip('/')}{path}", method="GET")
    try:
        with urllib.request.urlopen(request, timeout=timeout) as response:
            raw = response.read().decode("utf-8")
            payload: Any = json.loads(raw) if raw else {}
            return {"http_status": response.status, "summary": summarize_payload(payload)}
    except urllib.error.HTTPError as exc:
        return {"http_status": exc.code, "summary": {"status": "http_error"}}
    except (OSError, json.JSONDecodeError) as exc:
        return {"http_status": 0, "summary": {"status": "unavailable", "detail": str(exc)[:160]}}


def summarize_payload(payload: Any) -> dict[str, Any]:
    if isinstance(payload, dict):
        summary: dict[str, Any] = {"kind": "object", "keys": sorted(str(key) for key in payload.keys())[:30]}
        for key in ("status", "ready", "runtime", "capability_state", "exists", "collection_name"):
            value = payload.get(key)
            if isinstance(value, (str, int, float, bool)) or value is None:
                summary[key] = value
        return summary
    if isinstance(payload, list):
        return {"kind": "list", "count": len(payload)}
    return {"kind": type(payload).__name__}


def tool_presence() -> dict[str, Any]:
    presence: dict[str, Any] = {}
    for tool in TOOLS:
        path = shutil.which(tool)
        presence[tool] = {"present": path is not None}
    return presence


def latest_smoke_summary() -> dict[str, Any]:
    if not SMOKE_RESULT_DIR.is_dir():
        return {"present": False, "reason": "no smoke result directory"}
    candidates = sorted(SMOKE_RESULT_DIR.glob("operator-smoke-*.json"))
    if not candidates:
        return {"present": False, "reason": "no smoke result files"}
    path = candidates[-1]
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        return {"present": False, "reason": f"could not read latest smoke summary: {exc}"}
    if not isinstance(payload, dict):
        return {"present": False, "reason": "latest smoke result is not an object"}
    steps = payload.get("steps") if isinstance(payload.get("steps"), list) else []
    counts = payload.get("counts") if isinstance(payload.get("counts"), dict) else {}
    api_status = payload.get("api_status") if isinstance(payload.get("api_status"), dict) else {}
    return {
        "present": True,
        "file_name": path.name,
        "created_at_utc": payload.get("created_at_utc"),
        "repo_branch": payload.get("repo_branch"),
        "repo_head": payload.get("repo_head"),
        "mode": payload.get("mode"),
        "overall_status": payload.get("overall_status"),
        "step_count": len([step for step in steps if isinstance(step, dict)]),
        "failed_step_count": sum(1 for step in steps if isinstance(step, dict) and step.get("status") == "fail"),
        "counts": {
            "artifacts": counts.get("artifacts"),
            "documents": counts.get("documents"),
            "chunks": counts.get("chunks"),
            "evidence_items": counts.get("evidence_items"),
            "retrieval_items": counts.get("retrieval_items"),
        },
        "api_status": {
            "live_http_status": api_status.get("live_http_status"),
            "ready_http_status": api_status.get("ready_http_status"),
            "retrieval_preview_http_status": api_status.get("retrieval_preview_http_status"),
        },
    }


def build_bundle(api_base_url: str, timeout: float) -> dict[str, Any]:
    route_health = {
        name: {"path": path, **read_json_route(api_base_url, path, timeout)}
        for name, path in ROUTE_CHECKS
    }
    return {
        "schema_version": SCHEMA_VERSION,
        "created_at_utc": utc_now(),
        "repo": {
            "branch": git_value("branch", "--show-current"),
            "head": git_value("rev-parse", "HEAD"),
            "status": safe_git_status(),
        },
        "runtime_posture": {
            "active_api": "rust_gateway",
            "active_worker": "rust_worker",
            "active_web": "nextjs",
            "legacy_python_api": "archived_inactive",
            "legacy_python_worker": "archived_inactive",
            "celery_beat": "inactive_retired",
        },
        "route_health": route_health,
        "tool_presence": tool_presence(),
        "latest_smoke_result": latest_smoke_summary(),
        "exclusions": [
            ".env and .env backups",
            "credentials, tokens, cookies, and private keys",
            "raw runtime data and IGY6_DATA_ROOT contents",
            "raw artifact/document/chunk/evidence/report contents",
            "Docker volume data and service database dumps",
            "full logs and raw smoke result JSON",
            "local absolute paths where avoidable",
        ],
    }


def safety_findings(value: Any, path: str = "$") -> list[str]:
    findings: list[str] = []
    if isinstance(value, dict):
        for key, item in value.items():
            key_text = str(key)
            item_path = f"{path}.{key_text}"
            if SENSITIVE_RE.search(key_text):
                findings.append(f"sensitive-shaped key at {item_path}")
            findings.extend(safety_findings(item, item_path))
    elif isinstance(value, list):
        for index, item in enumerate(value):
            findings.extend(safety_findings(item, f"{path}[{index}]"))
    elif isinstance(value, str):
        if not path.startswith("$.exclusions") and SENSITIVE_RE.search(value):
            findings.append(f"sensitive-shaped value at {path}")
        if PRIVATE_PATH_RE.search(value):
            findings.append(f"private path value at {path}")
    return findings


def write_bundle(bundle: dict[str, Any], output_dir: Path) -> Path:
    output_dir.mkdir(parents=True, exist_ok=True)
    target = output_dir / f"igy6-diagnostics-{datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')}.json"
    target.write_text(json.dumps(bundle, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return target


def print_summary(bundle: dict[str, Any], target: Path | None) -> None:
    print("IGY6 Diagnostics Bundle MVP")
    print(f"schema_version: {bundle['schema_version']}")
    print(f"created_at_utc: {bundle['created_at_utc']}")
    print(f"repo_branch: {bundle['repo'].get('branch')}")
    print(f"repo_head: {bundle['repo'].get('head')}")
    print(f"repo_dirty: {bundle['repo']['status'].get('dirty')}")
    print("route_health:")
    for name, route in sorted(bundle["route_health"].items()):
        print(f"  {name}: HTTP {route.get('http_status')}")
    latest = bundle["latest_smoke_result"]
    print(f"latest_smoke_result_present: {latest.get('present')}")
    if latest.get("present"):
        print(f"latest_smoke_result_status: {latest.get('overall_status')}")
    if target is not None:
        try:
            display = target.resolve().relative_to(REPO_ROOT)
        except ValueError:
            display = target
        print(f"wrote: {display}")
    print("safety_validation: passed")


def main() -> int:
    parser = argparse.ArgumentParser(description="Create a safe IGY6 diagnostics bundle.")
    parser.add_argument("--api-base-url", default="http://127.0.0.1:8000")
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--timeout", type=float, default=3.0)
    parser.add_argument("--dry-run", action="store_true", help="Print diagnostics summary without writing a bundle.")
    args = parser.parse_args()

    bundle = build_bundle(args.api_base_url, args.timeout)
    unsafe_findings = safety_findings(bundle)
    if unsafe_findings:
        print("IGY6 Diagnostics Bundle MVP")
        print(f"schema_version: {bundle['schema_version']}")
        print("safety_validation: failed")
        print("unsafe_findings:")
        for finding in unsafe_findings:
            print(f"  {finding}")
        return 1
    target = None if args.dry_run else write_bundle(bundle, args.output_dir)
    print_summary(bundle, target)
    return 0


if __name__ == "__main__":
    sys.exit(main())
PY
