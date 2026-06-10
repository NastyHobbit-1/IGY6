# Web UI

The IGY6 web UI is the main browser interface for local runtime use.

## Open the UI

Start the stack, then open:

```text
http://127.0.0.1:13000
```

The web container maps local port `13000` to the application inside Docker.

## API connection

The UI talks to the Rust API gateway at:

```text
http://127.0.0.1:18000
```

The API gateway provides local runtime routes for health checks, evidence operations, work operations, settings, approvals, and related application functions.

## Main UI areas

The visible UI may include areas for:

- Chat and evidence-grounded answers
- Adding authorized information and sources
- Work status and processing
- Results, reports, and evidence records
- Settings and approvals
- Advanced diagnostics

Use only authorized source material. Evidence answers should be reviewed with their provenance trail.

## Troubleshooting

### Web page does not load

Check container status:

```powershell
docker compose -f infra/docker-compose.yml --env-file .env.test ps
```

Then verify the web endpoint:

```powershell
Invoke-WebRequest -Uri "http://127.0.0.1:13000" -UseBasicParsing | Select-Object StatusCode
```

### API is unreachable

Check:

```powershell
Invoke-RestMethod -Uri "http://127.0.0.1:18000/health/live"
Invoke-RestMethod -Uri "http://127.0.0.1:18000/health/ready"
```

### Environment file is missing

Create `.env` from `.env.example` and configure `IGY6_DATA_ROOT`.

### Container is unhealthy

Use Docker Compose status and logs to identify the service that is not ready.
