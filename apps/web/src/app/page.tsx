type HealthResponse = {
  status: string;
  checks?: Record<string, { status: string; detail?: string }>;
};

async function getHealth(): Promise<HealthResponse> {
  const baseUrl = process.env.API_BASE_URL ?? "http://api:8000";
  try {
    const response = await fetch(`${baseUrl}/health/ready`, {
      cache: "no-store"
    });
    return response.json();
  } catch (error) {
    return {
      status: "error",
      checks: {
        api: {
          status: "error",
          detail: error instanceof Error ? error.message : "Unknown error"
        }
      }
    };
  }
}

export default async function Home() {
  const health = await getHealth();
  const checks = health.checks ?? {};

  return (
    <main className="shell">
      <section className="header">
        <p className="eyebrow">Phase 0 skeleton</p>
        <h1>IGY6 Adaptive Intelligence System</h1>
        <p>
          Local-first service status for the approved Phase 0 foundation. This
          screen is not chat, ingestion, retrieval, prediction, or
          self-improvement.
        </p>
      </section>

      <section className="status">
        <div>
          <span className="label">Overall</span>
          <strong>{health.status}</strong>
        </div>
        <div className="grid">
          {Object.entries(checks).map(([name, check]) => (
            <article className="check" key={name}>
              <span>{name}</span>
              <strong data-state={check.status}>{check.status}</strong>
              {check.detail ? <small>{check.detail}</small> : null}
            </article>
          ))}
        </div>
      </section>
    </main>
  );
}
