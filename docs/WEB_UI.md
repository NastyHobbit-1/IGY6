# IGY6 Web UI

The IGY6 Web UI is the primary user interface for the local evidence and intelligence workspace. It provides an intuitive dashboard for data ingestion, analysis, and evidence-grounded decision support.

## Accessing the Web UI

Once the stack is running, open your browser and navigate to:

http://127.0.0.1:13000

## Connection to Backend

The web UI communicates with the Rust gateway/API service running at http://127.0.0.1:18000 (configurable via APP_PORT in .env).

## Main UI Areas

The interface includes tabs such as:
- Home
- Add Data
- Work
- Results
- Settings
- Advanced

## Adding Authorized Information

Use the Add Data section to securely ingest authorized documents, artifacts, and data sources. All processing remains local.

## Evidence-Grounded Chat and Retrieval

Interact with your ingested evidence through natural language queries. The system performs retrieval from vector and graph stores to deliver grounded responses.

## Provenance and Evidence Trail

Responses include detailed provenance information linking back to source artifacts, documents, and processing steps for full traceability and verification.

## Troubleshooting

- **Web container still starting**: Wait a few moments for the Next.js application to fully initialize.
- **API unreachable**: Verify the Rust gateway is running and check the health endpoints.
- **Missing .env or .env.test**: Copy `.env.example` to `.env` or `.env.test` and configure IGY6_DATA_ROOT.
- **Docker container unhealthy**: Use `docker compose ps` to check status.