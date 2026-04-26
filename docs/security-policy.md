# Security Policy

Phase 0 establishes the default safety posture that later phases must preserve.

## Defaults

- Local-first only.
- Services bind to `127.0.0.1` by default.
- Read-only by default.
- No real collectors in Phase 0.
- No browser automation in Phase 0.
- No external model calls in Phase 0.
- No hard-coded secrets.
- Placeholder credentials belong in `.env` only for local development.

## Required Future Enforcement

Every source access must require:

- Registered source.
- Explicit permission scope.
- Sensitivity label.
- Allowed operations.
- Audit event.

Every sensitive or system-changing action must require:

- Approval request.
- Approver decision.
- Audit event.
- Worker-side execution only after approval.

Sensitive actions include file writes, PC setting changes, router changes,
account actions, repository writes, website changes, sensitive data export, and
external model use with sensitive content.
