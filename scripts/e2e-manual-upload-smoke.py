#!/usr/bin/env python3
"""DIFF-124 local manual-upload smoke helper.

Default --check mode is non-mutating. --run explicitly creates harmless local
runtime records in the running IGY6 stack.
"""

from __future__ import annotations

import argparse
import base64
import json
import sys
import time
import urllib.error
import urllib.request
from typing import Any


PAYLOAD_TEXT = "IGY6 manual upload test. The secret test keyword is blue-raven-117."


class SmokeClient:
    def __init__(self, api_base_url: str, web_url: str, timeout: float) -> None:
        self.api_base_url = api_base_url.rstrip("/")
        self.web_url = web_url.rstrip("/")
        self.timeout = timeout

    def get(self, path: str) -> tuple[int, Any]:
        return self.request("GET", path)

    def post(self, path: str, payload: dict[str, Any]) -> tuple[int, Any]:
        return self.request("POST", path, payload)

    def request(self, method: str, path: str, payload: dict[str, Any] | None = None) -> tuple[int, Any]:
        data = None
        headers = {}
        if payload is not None:
            data = json.dumps(payload).encode("utf-8")
            headers["Content-Type"] = "application/json"
        request = urllib.request.Request(
            f"{self.api_base_url}{path}",
            data=data,
            headers=headers,
            method=method,
        )
        try:
            with urllib.request.urlopen(request, timeout=self.timeout) as response:
                raw = response.read().decode("utf-8")
                return response.status, json.loads(raw) if raw else {}
        except urllib.error.HTTPError as error:
            raw = error.read().decode("utf-8", errors="replace")
            try:
                payload_json: Any = json.loads(raw)
            except json.JSONDecodeError:
                payload_json = {"detail": raw}
            return error.code, payload_json

    def web_status(self) -> int:
        request = urllib.request.Request(self.web_url, method="GET")
        with urllib.request.urlopen(request, timeout=self.timeout) as response:
            response.read(1)
            return response.status


def pass_line(message: str) -> None:
    print(f"PASS {message}")


def fail_line(message: str) -> None:
    print(f"FAIL {message}")


def warn_line(message: str) -> None:
    print(f"WARN {message}")


def require_status(label: str, status: int, expected: set[int]) -> bool:
    if status in expected:
        pass_line(f"{label} returned HTTP {status}")
        return True
    fail_line(f"{label} returned HTTP {status}")
    return False


def preflight(client: SmokeClient) -> bool:
    ok = True
    try:
        status, _ = client.get("/health/live")
        ok = require_status("API live", status, {200}) and ok
    except OSError as exc:
        fail_line(f"API live unavailable: {exc}")
        ok = False

    try:
        status, _ = client.get("/health/ready")
        ok = require_status("API ready", status, {200}) and ok
    except OSError as exc:
        fail_line(f"API ready unavailable: {exc}")
        ok = False

    try:
        status = client.web_status()
        ok = require_status("Web UI", status, {200}) and ok
    except OSError as exc:
        fail_line(f"Web UI unavailable: {exc}")
        ok = False

    if not ok:
        print()
        print("Next diagnostics:")
        print("  scripts/runtime-smoke.sh --check")
        print("  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api")
        print("  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker")
        print("  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 web")
    return ok


def create_source(client: SmokeClient) -> tuple[str, str]:
    source_name = f"IGY6 Manual Upload Smoke {int(time.time())}"
    status, payload = client.post(
        "/sources",
        {
            "name": source_name,
            "source_type": "manual_upload",
            "location": "local smoke test",
            "owner_actor_id": "local-owner",
            "sensitivity": "internal",
            "trust_level": "unreviewed",
            "enabled": True,
            "metadata_json": {
                "smoke_test": "DIFF-124",
                "payload_keyword": "blue-raven-117",
            },
            "permission": {
                "scope_json": {"purpose": "DIFF-124 manual upload smoke"},
                "allowed_operations": ["read", "collect", "dry_run"],
                "external_model_policy": "blocked",
                "approval_required": True,
                "created_by_actor_id": "local-owner",
            },
        },
    )
    if status != 201:
        raise RuntimeError(f"source create failed HTTP {status}: {payload}")
    source_id = payload["id"]
    permissions = payload.get("permissions") or []
    if not permissions:
        raise RuntimeError("source create did not return a permission")
    permission_id = permissions[0]["id"]
    pass_line(f"created manual_upload source {source_id}")
    pass_line(f"created source permission {permission_id}")
    return source_id, permission_id


def approve_manual_upload(client: SmokeClient, source_id: str, permission_id: str) -> str:
    status, approval = client.post(
        "/approvals",
        {
            "request_type": "manual_upload_collection",
            "requested_by_actor_id": "local-owner",
            "request_payload_json": {
                "source_id": source_id,
                "source_permission_id": permission_id,
                "operation": "manual_upload_collection",
            },
        },
    )
    if status != 201:
        raise RuntimeError(f"approval request failed HTTP {status}: {approval}")
    approval_id = approval["id"]
    pass_line(f"created approval {approval_id}")

    status, decision = client.post(
        f"/approvals/{approval_id}/decision",
        {
            "status": "approved",
            "decided_by_actor_id": "local-owner",
            "decision_reason": "Approve DIFF-124 local manual upload smoke.",
        },
    )
    if status != 200:
        raise RuntimeError(f"approval decision failed HTTP {status}: {decision}")
    pass_line(f"approved manual upload approval {approval_id}")
    return approval_id


def upload_payload(
    client: SmokeClient,
    source_id: str,
    permission_id: str,
    approval_id: str,
) -> dict[str, Any]:
    encoded = base64.b64encode(PAYLOAD_TEXT.encode("utf-8")).decode("ascii")
    status, collection_run = client.post(
        "/collection-runs/manual-upload",
        {
            "source_id": source_id,
            "source_permission_id": permission_id,
            "approval_id": approval_id,
            "filename": "igy6-manual-upload-smoke.txt",
            "mime_type": "text/plain",
            "content_base64": encoded,
            "metadata_json": {
                "smoke_test": "DIFF-124",
                "payload_keyword": "blue-raven-117",
            },
            "requested_by_actor_id": "local-owner",
        },
    )
    if status != 201:
        raise RuntimeError(f"manual upload failed HTTP {status}: {collection_run}")
    pass_line(f"manual upload created collection run {collection_run.get('id')}")
    summary = collection_run.get("summary_json") or {}
    if summary.get("raw_artifact_ids"):
        pass_line(f"raw artifact created {summary['raw_artifact_ids'][0]}")
    else:
        warn_line("collection summary did not include raw_artifact_ids")
    if summary.get("normalization_work_item_id"):
        pass_line(f"normalization work item created {summary['normalization_work_item_id']}")
    else:
        warn_line("collection summary did not include normalization_work_item_id")
    return collection_run


def inspect_processing(client: SmokeClient, collection_run: dict[str, Any]) -> None:
    summary = collection_run.get("summary_json") or {}
    work_item_id = summary.get("normalization_work_item_id")

    status, work_items = client.get("/work-items")
    if status == 200 and isinstance(work_items, list):
        matching = [item for item in work_items if item.get("id") == work_item_id]
        if matching:
            pass_line(f"work item status is {matching[0].get('status')}")
        elif work_item_id:
            warn_line(f"work item {work_item_id} was not present in list response")
    else:
        warn_line(f"could not inspect work items: HTTP {status}")

    for label, path in (
        ("artifacts", "/artifacts"),
        ("documents", "/evidence/documents"),
        ("chunks", "/evidence/chunks"),
        ("evidence items", "/evidence/items"),
    ):
        status, payload = client.get(path)
        if status == 200 and isinstance(payload, list):
            pass_line(f"{label} endpoint returned {len(payload)} records")
        else:
            warn_line(f"{label} endpoint returned HTTP {status}")

    status, retrieval = client.post(
        "/chat/retrieval-preview",
        {
            "message": "Find blue-raven-117 in my uploaded evidence.",
            "limit": 5,
        },
    )
    if status != 200:
        warn_line(f"retrieval preview returned HTTP {status}")
        return
    context = retrieval.get("retrieval_context") or {}
    hits = context.get("hits") or retrieval.get("items") or []
    if hits:
        pass_line(f"retrieval preview returned {len(hits)} hit(s)")
    else:
        warn_line("retrieval preview returned no hits; worker processing may still be pending")


def run_smoke(client: SmokeClient) -> int:
    if not preflight(client):
        return 1
    try:
        source_id, permission_id = create_source(client)
        approval_id = approve_manual_upload(client, source_id, permission_id)
        collection_run = upload_payload(client, source_id, permission_id, approval_id)
        inspect_processing(client, collection_run)
    except RuntimeError as exc:
        fail_line(str(exc))
        return 1
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="DIFF-124 manual upload E2E smoke helper")
    parser.add_argument("--api-base-url", default="http://127.0.0.1:8000")
    parser.add_argument("--web-url", default="http://127.0.0.1:3000")
    parser.add_argument("--timeout", type=float, default=10.0)
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--check", action="store_true", help="non-mutating preflight only")
    mode.add_argument("--run", action="store_true", help="create local smoke records in the running stack")
    args = parser.parse_args()

    client = SmokeClient(args.api_base_url, args.web_url, args.timeout)
    if args.run:
        return run_smoke(client)

    print("Checklist-assisted smoke. Default --check mode does not create records.")
    print("Use --run only against a local stack where harmless smoke records are acceptable.")
    return 0 if preflight(client) else 1


if __name__ == "__main__":
    sys.exit(main())
