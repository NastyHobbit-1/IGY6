import sys
import unittest
from pathlib import Path

from fastapi import HTTPException

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from app.agent_actions import AgentActionExecuteRequest, AgentIntentRequest, classify_agent_intent, execute_agent_action
from app.config import Settings


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


if __name__ == "__main__":
    unittest.main()
