#!/usr/bin/env python3
"""Focused tests for the DIFF-119 route parity classification guard."""

from __future__ import annotations

import importlib.util
import json
import sys
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
ROUTE_PARITY_SCRIPT = REPO_ROOT / "scripts" / "rust-route-parity.py"
CLASSIFICATION = REPO_ROOT / "configs" / "legacy-fastapi-route-classification.json"
MANIFEST = REPO_ROOT / "configs" / "rust-cutover-manifest.json"


def load_route_parity_module():
    spec = importlib.util.spec_from_file_location("rust_route_parity", ROUTE_PARITY_SCRIPT)
    if spec is None or spec.loader is None:
        raise RuntimeError("could not load rust-route-parity.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules["rust_route_parity"] = module
    spec.loader.exec_module(module)
    return module


class RouteParityClassificationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.route_parity = load_route_parity_module()

    def test_missing_fastapi_routes_are_all_classified(self) -> None:
        fastapi = self.route_parity.fastapi_routes()
        rust = self.route_parity.rust_gateway_routes()
        missing = {
            route.key()
            for route in fastapi
            if not self.route_parity.route_set_contains(rust, route)
        }
        classification = json.loads(CLASSIFICATION.read_text(encoding="utf-8"))
        classified = {
            f"{entry['method']} {entry['path']}"
            for entry in classification["routes"]
        }
        self.assertEqual(missing, classified)

    def test_web_routes_do_not_require_fallback(self) -> None:
        rust = self.route_parity.rust_gateway_routes()
        web = self.route_parity.web_used_routes()
        web_fallback = [
            route.key()
            for route in web
            if not self.route_parity.route_set_contains(rust, route)
        ]
        self.assertEqual([], web_fallback)

    def test_dynamic_web_controls_are_explicitly_tracked(self) -> None:
        web = {route.key() for route in self.route_parity.web_used_routes()}
        self.assertIn("POST /analysis/patterns/{pattern_id}/review", web)
        self.assertIn("POST /approvals/{approval_id}/decision", web)
        self.assertIn("POST /reports/{report_id}/render", web)
        self.assertIn("POST /work-items/{work_item_id}/dispatch", web)

    def test_guard_accepts_current_manifest_and_classification(self) -> None:
        fastapi = self.route_parity.fastapi_routes()
        rust = self.route_parity.rust_gateway_routes()
        web = self.route_parity.web_used_routes()
        missing = sorted(
            route for route in fastapi if not self.route_parity.route_set_contains(rust, route)
        )
        web_fallback = sorted(
            route for route in web if not self.route_parity.route_set_contains(rust, route)
        )
        summary = {
            "fastapi_routes": len(fastapi),
            "rust_native_routes": len(rust),
            "web_used_routes": len(web),
            "fastapi_routes_missing_from_rust": len(missing),
            "web_routes_requiring_fallback": len(web_fallback),
            "fastapi_fallback_required": bool(missing or web_fallback),
        }
        manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        errors = self.route_parity._classification_errors(summary, missing, manifest)
        self.assertEqual([], errors)


if __name__ == "__main__":
    unittest.main()
