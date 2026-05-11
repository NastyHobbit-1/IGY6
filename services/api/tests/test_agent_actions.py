import sys
import unittest
from pathlib import Path
from unittest.mock import patch

from fastapi import HTTPException

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from app.agent_actions import (
    AgentActionExecuteRequest,
    AgentIntentRequest,
    classify_agent_intent,
    execute_agent_action,
    get_agent_capabilities,
)
from app.config import Settings
from app.models import Approval


class AgentActionRegistryTest(unittest.TestCase):
    def test_known_read_only_action_classification(self) -> None:
        intent = classify_agent_intent(AgentIntentRequest(message="show project health"))

        self.assertEqual(intent.proposed_action, "show_project_health")
        self.assertEqual(intent.action_type, "read_only")
        self.assertFalse(intent.approval_required)
        self.assertTrue(intent.executable_now)

    def test_unknown_action_execute_rejected(self) -> None:
        with self.assertRaises(HTTPException) as raised:
            execute_agent_action(
                "run_any_shell",
                AgentActionExecuteRequest(parameters={}),
                db=object(),  # type: ignore[arg-type]
                settings=Settings(),
            )

        self.assertEqual(raised.exception.status_code, 404)

    def test_arbitrary_shell_request_rejected_by_intent(self) -> None:
        intent = classify_agent_intent(AgentIntentRequest(message="run rm -rf /"))

        self.assertIsNone(intent.proposed_action)
        self.assertEqual(intent.action_type, "unknown")
        self.assertFalse(intent.executable_now)
        self.assertIn("Arbitrary shell", intent.reason or "")

    def test_approval_required_action_blocked_without_approval(self) -> None:
        with self.assertRaises(HTTPException) as raised:
            execute_agent_action(
                "start_stack",
                AgentActionExecuteRequest(parameters={}),
                db=object(),  # type: ignore[arg-type]
                settings=Settings(),
            )

        self.assertEqual(raised.exception.status_code, 403)
        self.assertIn("requires approval", str(raised.exception.detail))

    def test_capabilities_include_runtime_and_stack_control_reason(self) -> None:
        capabilities = get_agent_capabilities()
        by_name = {action.name: action for action in capabilities.actions}

        self.assertIn("show_project_health", by_name)
        self.assertIn("start_stack", by_name)
        self.assertTrue(by_name["show_project_health"].executable_in_api_runtime)
        self.assertTrue(by_name["start_stack"].script_backed)
        self.assertIn("scripts/run.sh", by_name["start_stack"].required_scripts)
        self.assertIsInstance(capabilities.runtime.docker_cli_available, bool)

    def test_approved_stack_action_blocked_when_runtime_not_capable(self) -> None:
        class FakeDb:
            def get(self, model: object, approval_id: str) -> Approval | None:
                return Approval(
                    id=approval_id,
                    request_type="agent_action",
                    status="approved",
                    requested_by_actor_id="local-owner",
                    request_payload_json={"action_name": "start_stack", "parameters": {}},
                )

        with patch("app.agent_actions.which", return_value=None):
            with self.assertRaises(HTTPException) as raised:
                execute_agent_action(
                    "start_stack",
                    AgentActionExecuteRequest(parameters={}, approval_id="approval-1"),
                    db=FakeDb(),  # type: ignore[arg-type]
                    settings=Settings(),
                )

        self.assertEqual(raised.exception.status_code, 409)
        self.assertIn("not executable", str(raised.exception.detail))


if __name__ == "__main__":
    unittest.main()
