#!/usr/bin/env python3
"""DIFF-125 processing status smoke helper.

Checks already-running local services and processing status without mutating
runtime state.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import urllib.error
import urllib.request
from collections import Counter
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[1]
COMPOSE_FILE = REPO_ROOT / "infra" / "docker-compose.yml"


def pass_line(message: str) -> None:
    print(f"PASS {message}")


def fail_line(message: str) -> None:
    print(f"FAIL {message}")


def warn_line(message: str) -> None:
    print(f"WARN {message}")


def read_env(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    if not path.is_file():
        return values
    for line in path.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#") or "=" not in stripped:
            continue
        key, value = stripped.split("=", 1)
        values[key] = value
    return values


class ProcessingSmoke:
    def __init__(self, env_file: Path, api_base_url: str, timeout: float) -> None:
        self.env_file = env_file
        self.api_base_url = api_base_url.rstrip("/")
        self.timeout = timeout
        self.failures = 0
        self.env = read_env(env_file)

    def compose(self, *args: str) -> subprocess.CompletedProcess[str]:
        command = [
            "docker",
            "compose",
            "-f",
            str(COMPOSE_FILE),
            "--env-file",
            str(self.env_file),
            *args,
        ]
        return subprocess.run(command, text=True, capture_output=True, check=False)

    def check(self, condition: bool, success: str, failure: str) -> None:
        if condition:
            pass_line(success)
        else:
            fail_line(failure)
            self.failures += 1

    def check_compose_config(self) -> None:
        result = self.compose("config")
        self.check(
            result.returncode == 0,
            f"docker compose config is valid using {self.env_file}",
            f"docker compose config failed using {self.env_file}",
        )

    def running_services(self) -> set[str]:
        result = self.compose("ps", "--services", "--filter", "status=running")
        if result.returncode != 0:
            fail_line("docker compose ps failed")
            self.failures += 1
            return set()
        services = {line.strip() for line in result.stdout.splitlines() if line.strip()}
        if not services:
            fail_line("no running compose services found")
            self.failures += 1
        return services

    def check_service_set(self, services: set[str]) -> None:
        for service in ("worker", "redis", "postgres", "qdrant", "api", "web"):
            self.check(
                service in services,
                f"service {service} is running",
                f"service {service} is not running",
            )

    def check_redis(self) -> None:
        result = self.compose("exec", "-T", "redis", "redis-cli", "ping")
        self.check(
            result.returncode == 0 and "PONG" in result.stdout,
            "Redis responds to PING",
            "Redis did not respond to PING",
        )

    def check_postgres(self) -> None:
        user = self.env.get("POSTGRES_USER", "adaptive")
        database = self.env.get("POSTGRES_DB", "adaptive_intelligence")
        result = self.compose("exec", "-T", "postgres", "pg_isready", "-U", user, "-d", database)
        self.check(
            result.returncode == 0,
            "Postgres responds to pg_isready",
            "Postgres did not respond to pg_isready",
        )

    def get_json(self, path: str) -> tuple[int, Any]:
        request = urllib.request.Request(f"{self.api_base_url}{path}", method="GET")
        try:
            with urllib.request.urlopen(request, timeout=self.timeout) as response:
                raw = response.read().decode("utf-8")
                return response.status, json.loads(raw) if raw else {}
        except urllib.error.HTTPError as error:
            raw = error.read().decode("utf-8", errors="replace")
            try:
                payload: Any = json.loads(raw)
            except json.JSONDecodeError:
                payload = {"detail": raw}
            return error.code, payload
        except OSError as exc:
            return 0, {"detail": str(exc)}

    def check_api_processing_views(self) -> None:
        status, ready = self.get_json("/health/ready")
        self.check(status == 200, "API ready endpoint responded", f"API ready failed: {ready}")

        status, work_items = self.get_json("/work-items")
        if status == 200 and isinstance(work_items, list):
            counts = Counter(str(item.get("status", "unknown")) for item in work_items)
            pass_line(f"work items inspectable: {dict(sorted(counts.items()))}")
        else:
            fail_line(f"work items not inspectable: HTTP {status} {work_items}")
            self.failures += 1

        status, vector = self.get_json("/memory/vector/chunks")
        if status == 200 and isinstance(vector, dict):
            pass_line(
                "Qdrant vector status inspectable: "
                f"collection={vector.get('collection_name')} exists={vector.get('exists')}"
            )
        else:
            fail_line(f"Qdrant vector status not inspectable: HTTP {status} {vector}")
            self.failures += 1

    def run(self) -> int:
        self.check_compose_config()
        services = self.running_services()
        self.check_service_set(services)
        if "redis" in services:
            self.check_redis()
        if "postgres" in services:
            self.check_postgres()
        self.check_api_processing_views()
        if self.failures:
            print()
            print("Next diagnostics:")
            print("  scripts/runtime-smoke.sh --check")
            print("  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 worker")
            print("  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 api")
            print("  docker compose -f infra/docker-compose.yml --env-file .env logs -f --tail=200 redis")
            return 1
        return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="DIFF-125 worker/processing status smoke")
    default_env = REPO_ROOT / ".env"
    if not default_env.is_file():
        default_env = REPO_ROOT / ".env.example"
    parser.add_argument("--env-file", type=Path, default=default_env)
    parser.add_argument("--api-base-url", default="http://127.0.0.1:8000")
    parser.add_argument("--timeout", type=float, default=8.0)
    args = parser.parse_args()

    return ProcessingSmoke(args.env_file, args.api_base_url, args.timeout).run()


if __name__ == "__main__":
    sys.exit(main())
