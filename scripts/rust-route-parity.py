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
    pattern = re.compile(r'\("([A-Z]+)",\s*"([^"]+)"\)\s*=>')
    return {Route(method, path) for method, path in pattern.findall(source)}


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
        manifest_fallback = bool(manifest.get("fastapi_fallback_required"))
        route_parity = manifest.get("route_parity", {})
        if manifest_fallback != summary["fastapi_fallback_required"]:
            raise SystemExit(
                "manifest fastapi_fallback_required does not match route parity"
            )
        if route_parity.get("rust_native_routes") != summary["rust_native_routes"]:
            raise SystemExit("manifest rust_native_routes is stale")
        if route_parity.get("status") == "complete" and summary["fastapi_fallback_required"]:
            raise SystemExit("manifest route_parity cannot be complete while fallback is required")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
