#!/usr/bin/env python3
"""DIFF-105 route parity guard for the Rust gateway cutover.

The script uses repository source files only. It does not read runtime data,
private storage, .env contents, databases, or network services.
"""

from __future__ import annotations

import argparse
import ast
import json
import re
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
FASTAPI_APP = REPO_ROOT / "services" / "api" / "app"
GATEWAY_LIB = REPO_ROOT / "crates" / "igy6-gateway" / "src" / "lib.rs"
WEB_ROOT = REPO_ROOT / "apps" / "web"
MANIFEST = REPO_ROOT / "configs" / "rust-cutover-manifest.json"
CLASSIFICATION = REPO_ROOT / "configs" / "legacy-fastapi-route-classification.json"
CLASSIFICATION_BUCKETS = {
    "active_parity_required",
    "intentional_legacy_fallback",
    "retireable_unused",
    "duplicate_or_superseded",
    "unsafe_to_migrate_now",
}


@dataclass(frozen=True, order=True)
class Route:
    method: str
    path: str

    def key(self) -> str:
        return f"{self.method} {self.path}"


def _constant_string(node: ast.AST) -> str | None:
    if isinstance(node, ast.Constant) and isinstance(node.value, str):
        return node.value
    return None


def fastapi_routes() -> set[Route]:
    routes: set[Route] = {Route("GET", "/")}
    for path in sorted(FASTAPI_APP.glob("*.py")):
        source = path.read_text(encoding="utf-8")
        tree = ast.parse(source)
        prefix = ""
        for node in tree.body:
            if isinstance(node, ast.Assign) and isinstance(node.value, ast.Call):
                if getattr(node.value.func, "id", "") == "APIRouter":
                    for keyword in node.value.keywords:
                        if keyword.arg == "prefix":
                            prefix = _constant_string(keyword.value) or ""
            if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            for decorator in node.decorator_list:
                if not isinstance(decorator, ast.Call):
                    continue
                func = decorator.func
                if not (
                    isinstance(func, ast.Attribute)
                    and isinstance(func.value, ast.Name)
                    and func.value.id == "router"
                ):
                    continue
                method = func.attr.upper()
                if method not in {"GET", "POST", "PUT", "PATCH", "DELETE"}:
                    continue
                route_path = ""
                if decorator.args:
                    route_path = _constant_string(decorator.args[0]) or ""
                routes.add(Route(method, f"{prefix}{route_path}"))
    return routes


def rust_gateway_routes() -> set[Route]:
    source = GATEWAY_LIB.read_text(encoding="utf-8")
    match_arm_pattern = re.compile(r'\("([A-Z]+)",\s*"([^"]+)"\)\s*=>')
    routes = {
        Route(method, path)
        for method, path in match_arm_pattern.findall(source)
    }
    registry_match = re.search(
        r"pub const RUST_NATIVE_ROUTES:.*?&\[(?P<body>.*?)\];",
        source,
        flags=re.DOTALL,
    )
    if registry_match:
        declared_pattern = re.compile(r'\("([A-Z]+)",\s*"([^"]+)"\),')
        routes.update(
            Route(method, path)
            for method, path in declared_pattern.findall(registry_match.group("body"))
        )
    return routes


def web_used_routes() -> set[Route]:
    routes: set[Route] = set()
    method_by_function = {"getJson": "GET", "postJson": "POST"}
    literal_route = re.compile(r'(?P<fn>getJson|postJson)<?[^("]*\(\s*"(?P<path>/[^"]+)"')
    fetch_route = re.compile(r'fetch\([^`\n]*`[^`]*\$\{apiBaseUrl\}(?P<path>/[^`]+)`')
    local_fetch = re.compile(r'fetch\(\s*"(?P<path>/api/[^"]+)"')
    for path in sorted(WEB_ROOT.rglob("*.ts*")):
        source = path.read_text(encoding="utf-8")
        for match in literal_route.finditer(source):
            routes.add(Route(method_by_function[match.group("fn")], match.group("path")))
        for match in fetch_route.finditer(source):
            route_path = match.group("path")
            method = "POST" if "method: \"POST\"" in source[match.end() : match.end() + 240] else "GET"
            routes.add(Route(method, route_path))
        for match in local_fetch.finditer(source):
            route_path = match.group("path").removeprefix("/api")
            method = "POST" if "method: \"POST\"" in source[match.end() : match.end() + 240] else "GET"
            routes.add(Route(method, route_path))
    return routes


def route_matches(pattern: str, concrete: str) -> bool:
    if pattern == concrete:
        return True
    pattern_parts = pattern.strip("/").split("/")
    concrete_parts = concrete.strip("/").split("/")
    if len(pattern_parts) != len(concrete_parts):
        return False
    return all(
        pattern_part == concrete_part
        or (pattern_part.startswith("{") and pattern_part.endswith("}"))
        or (concrete_part.startswith("[") and concrete_part.endswith("]"))
        for pattern_part, concrete_part in zip(pattern_parts, concrete_parts)
    )


def route_set_contains(routes: set[Route], route: Route) -> bool:
    return any(
        candidate.method == route.method and route_matches(candidate.path, route.path)
        for candidate in routes
    )


def _route_from_classification(entry: dict[str, object]) -> Route:
    method = entry.get("method")
    path = entry.get("path")
    if not isinstance(method, str) or not isinstance(path, str):
        raise ValueError("classification route entries require string method and path")
    return Route(method, path)


def _classification_errors(
    summary: dict[str, object],
    missing_from_rust: list[Route],
    manifest: dict[str, object],
) -> list[str]:
    errors: list[str] = []
    if summary["web_routes_requiring_fallback"] != 0:
        errors.append("web_routes_requiring_fallback must remain 0 after DIFF-118")

    if not CLASSIFICATION.exists():
        errors.append(f"missing route classification file: {CLASSIFICATION}")
        return errors

    classification = json.loads(CLASSIFICATION.read_text(encoding="utf-8"))
    routes = classification.get("routes")
    if not isinstance(routes, list):
        errors.append("classification routes must be a list")
        return errors

    missing_keys = {route.key() for route in missing_from_rust}
    classified_keys: set[str] = set()
    bucket_counts = {bucket: 0 for bucket in CLASSIFICATION_BUCKETS}

    for entry in routes:
        if not isinstance(entry, dict):
            errors.append("classification route entries must be objects")
            continue
        try:
            route = _route_from_classification(entry)
        except ValueError as exc:
            errors.append(str(exc))
            continue
        key = route.key()
        if key in classified_keys:
            errors.append(f"duplicate classified route: {key}")
        classified_keys.add(key)

        if entry.get("rust_status") != "missing_from_rust":
            errors.append(f"classified route {key} must have rust_status missing_from_rust")
        bucket = entry.get("classification")
        if bucket not in CLASSIFICATION_BUCKETS:
            errors.append(f"classified route {key} has invalid classification {bucket!r}")
        elif isinstance(bucket, str):
            bucket_counts[bucket] += 1

        for field_name in (
            "python_module",
            "python_handler",
            "reason",
            "migration_risk",
            "retirement_condition",
            "recommended_future_diff",
        ):
            if not isinstance(entry.get(field_name), str) or not entry.get(field_name):
                errors.append(f"classified route {key} is missing {field_name}")
        for field_name in ("used_by_apps_web", "used_by_scripts_tests_docs"):
            if not isinstance(entry.get(field_name), bool):
                errors.append(f"classified route {key} is missing boolean {field_name}")

    unclassified = sorted(missing_keys - classified_keys)
    unexpected = sorted(classified_keys - missing_keys)
    if unclassified:
        errors.append("unclassified missing FastAPI routes: " + ", ".join(unclassified))
    if unexpected:
        errors.append("classification contains routes not missing from Rust: " + ", ".join(unexpected))

    recorded_counts = classification.get("classification_counts")
    if recorded_counts != dict(sorted(bucket_counts.items())) and recorded_counts != bucket_counts:
        errors.append("classification_counts do not match classified routes")

    recorded_parity = classification.get("route_parity", {})
    if not isinstance(recorded_parity, dict):
        errors.append("classification route_parity must be an object")
    else:
        parity_checks = {
            "fastapi_routes": summary["fastapi_routes"],
            "rust_native_routes": summary["rust_native_routes"],
            "fastapi_routes_missing_from_rust": summary["fastapi_routes_missing_from_rust"],
            "web_used_routes": summary["web_used_routes"],
            "web_routes_requiring_fallback": summary["web_routes_requiring_fallback"],
        }
        for key, value in parity_checks.items():
            if recorded_parity.get(key) != value:
                errors.append(f"classification route_parity {key} is stale")

    requires_legacy = (
        bucket_counts["intentional_legacy_fallback"] > 0
        or bucket_counts["unsafe_to_migrate_now"] > 0
        or bool(missing_from_rust)
    )
    if bool(classification.get("fastapi_fallback_required")) != requires_legacy:
        errors.append("classification fastapi_fallback_required is stale")
    if classification.get("rust_only_claim_allowed") is not (not requires_legacy):
        errors.append("classification rust_only_claim_allowed is stale")

    route_parity = manifest.get("route_parity", {})
    if not isinstance(route_parity, dict):
        errors.append("manifest route_parity must be an object")
        route_parity = {}

    if bool(manifest.get("fastapi_fallback_required")) != summary["fastapi_fallback_required"]:
        errors.append("manifest fastapi_fallback_required does not match route parity")
    if route_parity.get("rust_native_routes") != summary["rust_native_routes"]:
        errors.append("manifest rust_native_routes is stale")
    if route_parity.get("fastapi_routes_missing_from_rust") != summary["fastapi_routes_missing_from_rust"]:
        errors.append("manifest fastapi_routes_missing_from_rust is stale")
    if route_parity.get("web_routes_requiring_fallback") != summary["web_routes_requiring_fallback"]:
        errors.append("manifest web_routes_requiring_fallback is stale")
    if route_parity.get("status") == "complete" and summary["fastapi_fallback_required"]:
        errors.append("manifest route_parity cannot be complete while fallback is required")

    target_architecture = str(manifest.get("target_architecture", ""))
    operational_status = str(manifest.get("operational_status", ""))
    if requires_legacy and (
        "rust-only" in target_architecture
        or "rust-only" in operational_status
        or route_parity.get("status") == "complete"
        or not bool(manifest.get("fastapi_fallback_required"))
    ):
        errors.append("manifest must not claim Rust-only while legacy fallback remains")

    return errors


def main() -> int:
    parser = argparse.ArgumentParser(description="Check IGY6 Rust/FastAPI route parity.")
    parser.add_argument("--check", action="store_true", help="validate manifest against route parity")
    parser.add_argument("--json", action="store_true", help="print JSON summary")
    args = parser.parse_args()

    fastapi = fastapi_routes()
    rust = rust_gateway_routes()
    web = web_used_routes()
    missing_from_rust = sorted(route for route in fastapi if not route_set_contains(rust, route))
    web_requires_fallback = sorted(route for route in web if not route_set_contains(rust, route))

    summary = {
        "fastapi_routes": len(fastapi),
        "rust_native_routes": len(rust),
        "web_used_routes": len(web),
        "fastapi_routes_missing_from_rust": len(missing_from_rust),
        "web_routes_requiring_fallback": len(web_requires_fallback),
        "fastapi_fallback_required": bool(missing_from_rust or web_requires_fallback),
        "rust_native": [route.key() for route in sorted(rust)],
        "missing_from_rust": [route.key() for route in missing_from_rust],
        "web_requires_fallback": [route.key() for route in web_requires_fallback],
    }

    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(
            "Route parity: "
            f"fastapi={summary['fastapi_routes']} "
            f"rust_native={summary['rust_native_routes']} "
            f"web_used={summary['web_used_routes']} "
            f"missing_from_rust={summary['fastapi_routes_missing_from_rust']} "
            f"web_requires_fallback={summary['web_routes_requiring_fallback']}"
        )

    if args.check:
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        errors = _classification_errors(summary, missing_from_rust, manifest)
        if errors:
            raise SystemExit("\n".join(errors))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
