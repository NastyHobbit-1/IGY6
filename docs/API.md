# Runtime API

IGY6 exposes its runtime API through the Rust gateway. The web UI uses the same local gateway endpoints that operators can verify directly.

## Default local endpoints

| Service | URL |
|---|---|
| Web UI | `http://127.0.0.1:13000` |
| Rust API gateway | `http://127.0.0.1:18000` |

## Health endpoints

Use these endpoints to confirm the gateway is running and ready:

```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:18000/health/live"
Invoke-RestMethod -Uri "http://127.0.0.1:18000/health/ready"
```

Expected result: both endpoints return an `ok` status. The ready endpoint should identify the Rust gateway as the active gateway.

## Runtime API role

The API coordinates local runtime functions:

- health checks
- source and artifact operations
- evidence retrieval
- evidence-grounded answer requests
- work item and report operations when available
- settings and approval flows when available

The web UI is the preferred interface for normal use. Direct API calls are mainly for verification, integration, and troubleshooting.

## Public-use boundary

Use the API only for authorized local runtime operations. Do not commit local environment files, runtime data, logs, database files, exports, or generated cache files.
