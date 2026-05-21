#!/usr/bin/env python3
from __future__ import annotations

import argparse
import ast
import json
import re
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
FASTAPI = ROOT / "services" / "api" / "app"
GATEWAY = ROOT / "crates" / "igy6-gateway" / "src" / "lib.rs"
WEB = ROOT / "apps" / "web"
MANIFEST = ROOT / "configs" / "rust-cutover-manifest.json"
CLASSIFICATION = ROOT / "configs" / "legacy-fastapi-route-classification.json"
RETIRED = {("GET", "/")}
EXPLICIT_WEB = {
    ("POST", "/analysis/patterns/{pattern_id}/review"),
    ("POST", "/approvals/{approval_id}/decision"),
    ("POST", "/reports/{report_id}/render"),
    ("POST", "/work-items/{work_item_id}/dispatch"),
}
BUCKETS = {
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


def const_str(node: ast.AST) -> str | None:
    return node.value if isinstance(node, ast.Constant) and isinstance(node.value, str) else None


def fastapi_routes() -> set[Route]:
    routes = {Route("GET", "/")}
    for file in sorted(FASTAPI.glob("*.py")):
        tree = ast.parse(file.read_text(encoding="utf-8"))
        prefix = ""
        for node in tree.body:
            if isinstance(node, ast.Assign) and isinstance(node.value, ast.Call):
                if getattr(node.value.func, "id", "") == "APIRouter":
                    for kw in node.value.keywords:
                        if kw.arg == "prefix":
                            prefix = const_str(kw.value) or ""
            if not isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                continue
            for dec in node.decorator_list:
                if not isinstance(dec, ast.Call) or not isinstance(dec.func, ast.Attribute):
                    continue
                if not isinstance(dec.func.value, ast.Name) or dec.func.value.id != "router":
                    continue
                method = dec.func.attr.upper()
                if method in {"GET", "POST", "PUT", "PATCH", "DELETE"}:
                    route_path = const_str(dec.args[0]) if dec.args else ""
                    routes.add(Route(method, f"{prefix}{route_path or ''}"))
    return routes


def rust_routes() -> set[Route]:
    text = GATEWAY.read_text(encoding="utf-8")
    routes = {Route(m, p) for m, p in re.findall(r'\("([A-Z]+)",\s*"([^"]+)"\)\s*=>', text)}
    match = re.search(r"RUST_NATIVE_ROUTES:.*?&\[(.*?)\];", text, re.S)
    if match:
        routes.update(Route(m, p) for m, p in re.findall(r'\("([A-Z]+)",\s*"([^"]+)"\),', match.group(1)))
    return routes


def web_routes() -> set[Route]:
    routes = {Route(m, p) for m, p in EXPLICIT_WEB}
    literal = re.compile(r'(?P<fn>getJson|postJson)<?[^("]*\(\s*"(?P<path>/[^"]+)"')
    fetch_remote = re.compile(r'fetch\([^`\n]*`[^`]*\$\{apiBaseUrl\}(?P<path>/[^`]+)`')
    fetch_local = re.compile(r'fetch\(\s*"(?P<path>/api/[^"]+)"')
    for file in sorted(WEB.rglob("*.ts*")):
        text = file.read_text(encoding="utf-8")
        for hit in literal.finditer(text):
            routes.add(Route("POST" if hit.group("fn") == "postJson" else "GET", hit.group("path")))
        for hit in fetch_remote.finditer(text):
            method = "POST" if "method: \"POST\"" in text[hit.end():hit.end()+240] else "GET"
            routes.add(Route(method, hit.group("path")))
        for hit in fetch_local.finditer(text):
            method = "POST" if "method: \"POST\"" in text[hit.end():hit.end()+240] else "GET"
            routes.add(Route(method, hit.group("path").removeprefix("/api")))
    return routes


def matches(pattern: str, concrete: str) -> bool:
    if pattern == concrete:
        return True
    a = pattern.strip("/").split("/")
    b = concrete.strip("/").split("/")
    return len(a) == len(b) and all(x == y or (x.startswith("{") and x.endswith("}")) or (y.startswith("[") and y.endswith("]")) for x, y in zip(a, b))


def contains(routes: set[Route], route: Route) -> bool:
    return any(r.method == route.method and matches(r.path, route.path) for r in routes)


def check(summary: dict[str, object], missing: list[Route]) -> list[str]:
    errors: list[str] = []
    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    classification = json.loads(CLASSIFICATION.read_text(encoding="utf-8"))
    route_parity = classification.get("route_parity", {})
    manifest_parity = manifest.get("route_parity", {})
    for key in ("fastapi_routes", "rust_native_routes", "fastapi_routes_missing_from_rust", "web_used_routes", "web_routes_requiring_fallback"):
        if route_parity.get(key) != summary[key]:
            errors.append(f"classification route_parity {key} is stale")
    for key in ("rust_native_routes", "fastapi_routes_missing_from_rust", "web_routes_requiring_fallback"):
        if manifest_parity.get(key) != summary[key]:
            errors.append(f"manifest {key} is stale")
    required = bool(missing or summary["web_routes_requiring_fallback"])
    if classification.get("fastapi_fallback_required") != required:
        errors.append("classification fastapi_fallback_required is stale")
    if manifest.get("fastapi_fallback_required") != required:
        errors.append("manifest fastapi_fallback_required does not match route parity")
    counts = {bucket: 0 for bucket in BUCKETS}
    classified = set()
    for entry in classification.get("routes", []):
        if not isinstance(entry, dict):
            errors.append("classification route entries must be objects")
            continue
        method = entry.get("method")
        path = entry.get("path")
        bucket = entry.get("classification")
        if isinstance(method, str) and isinstance(path, str):
            classified.add(f"{method} {path}")
        if bucket in counts:
            counts[bucket] += 1
        else:
            errors.append(f"invalid classification bucket {bucket!r}")
    if classification.get("classification_counts") not in (counts, dict(sorted(counts.items()))):
        errors.append("classification_counts do not match classified routes")
    missing_keys = {route.key() for route in missing}
    if missing_keys - classified:
        errors.append("unclassified missing FastAPI routes: " + ", ".join(sorted(missing_keys - classified)))
    if classified - missing_keys:
        errors.append("classification contains routes not missing from Rust: " + ", ".join(sorted(classified - missing_keys)))
    if classification.get("rust_only_claim_allowed") is not (not required):
        errors.append("classification rust_only_claim_allowed is stale")
    if manifest_parity.get("status") == "complete" and required:
        errors.append("manifest route_parity cannot be complete while fallback is required")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()
    fastapi = fastapi_routes()
    rust = rust_routes()
    web = web_routes()
    missing = sorted(r for r in fastapi if (r.method, r.path) not in RETIRED and not contains(rust, r))
    web_missing = sorted(r for r in web if not contains(rust, r))
    summary = {
        "fastapi_routes": len(fastapi),
        "rust_native_routes": len(rust),
        "web_used_routes": len(web),
        "fastapi_routes_missing_from_rust": len(missing),
        "web_routes_requiring_fallback": len(web_missing),
        "fastapi_fallback_required": bool(missing or web_missing),
        "rust_native": [r.key() for r in sorted(rust)],
        "missing_from_rust": [r.key() for r in missing],
        "retired_from_parity": [f"{m} {p}" for m, p in sorted(RETIRED)],
        "web_requires_fallback": [r.key() for r in web_missing],
    }
    if args.json:
        print(json.dumps(summary, indent=2, sort_keys=True))
    else:
        print(f"Route parity: fastapi={summary['fastapi_routes']} rust_native={summary['rust_native_routes']} web_used={summary['web_used_routes']} missing_from_rust={summary['fastapi_routes_missing_from_rust']} web_requires_fallback={summary['web_routes_requiring_fallback']}")
    if args.check:
        if errors := check(summary, missing):
            raise SystemExit("\n".join(errors))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
