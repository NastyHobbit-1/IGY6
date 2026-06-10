# IGY6 Web UI

The web UI is the primary user interface for the local evidence and intelligence workspace.

## Accessing the UI
Open your browser to http://127.0.0.1:13000 after starting the stack.

## API Connection
The UI connects to the Rust gateway/API service at http://127.0.0.1:18000 by default.

## Visible Tabs and Areas
- Chat: Evidence-grounded questions and retrieval.
- Data: Add authorized information and sources.
- Work: Processing status.
- Settings: Approvals and configuration.
- Advanced/More: Diagnostics.

## Workflow Highlights
Authorized information is added through the Data tab. Evidence is reviewed with full provenance trails. Chat provides grounded responses.

## Troubleshooting
- Web container starting: Wait for Docker Compose to complete.
- API unreachable: Check health endpoints and Docker logs.
- Missing .env: Copy from .env.example and set IGY6_DATA_ROOT.
- Unhealthy container: Use docker compose ps and restart.