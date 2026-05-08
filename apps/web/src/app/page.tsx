type HealthResponse = {
  status: string;
  checks?: Record<string, { status: string; detail?: string }>;
};

type SourcePermission = {
  id: string;
  allowed_operations: string[];
  approval_required: boolean;
  external_model_policy: string;
};

type SourceRecord = {
  id: string;
  name: string;
  source_type: string;
  sensitivity: string;
  trust_level: string;
  enabled: boolean;
  permissions?: SourcePermission[];
};

type CollectionRunRecord = {
  id: string;
  source_id: string | null;
  status: string;
  dry_run: boolean;
  requested_by_actor_id: string;
  created_at: string;
  summary_json: Record<string, unknown>;
};

type RawArtifactRecord = {
  id: string;
  source_id: string | null;
  collection_run_id: string | null;
  content_hash: string;
  mime_type: string | null;
  size_bytes: number | null;
  created_at: string;
};

type ApiResult<T> = {
  data: T;
  error: string | null;
};

async function getJson<T>(path: string, fallback: T): Promise<ApiResult<T>> {
  const baseUrl = process.env.API_BASE_URL ?? "http://api:8000";

  try {
    const response = await fetch(`${baseUrl}${path}`, {
      cache: "no-store"
    });
    if (!response.ok) {
      return { data: fallback, error: `${response.status} ${response.statusText}` };
    }
    return { data: (await response.json()) as T, error: null };
  } catch (error) {
    return {
      data: fallback,
      error: error instanceof Error ? error.message : "Unknown error"
    };
  }
}

function compactId(value: string | null): string {
  if (!value) {
    return "none";
  }
  return value.length > 12 ? `${value.slice(0, 8)}...` : value;
}

function formatDate(value: string): string {
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) {
    return value;
  }
  return date.toLocaleString("en-US", {
    dateStyle: "medium",
    timeStyle: "short"
  });
}

function formatBytes(value: number | null): string {
  if (value === null) {
    return "unknown";
  }
  if (value < 1024) {
    return `${value} B`;
  }
  if (value < 1024 * 1024) {
    return `${(value / 1024).toFixed(1)} KB`;
  }
  return `${(value / (1024 * 1024)).toFixed(1)} MB`;
}

function StatusPill({ state }: { state: string }) {
  return <span className="pill" data-state={state}>{state}</span>;
}

function EmptyState({ label }: { label: string }) {
  return <p className="empty">{label}</p>;
}

export default async function Home() {
  const [health, sources, collectionRuns, artifacts] = await Promise.all([
    getJson<HealthResponse>("/health/ready", { status: "error" }),
    getJson<SourceRecord[]>("/sources", []),
    getJson<CollectionRunRecord[]>("/collection-runs", []),
    getJson<RawArtifactRecord[]>("/artifacts", [])
  ]);

  const checks = health.data.checks ?? {};
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentArtifacts = artifacts.data.slice(0, 6);

  return (
    <main className="shell">
      <section className="header">
        <div>
          <p className="eyebrow">Local inventory</p>
          <h1>IGY6 Adaptive Intelligence System</h1>
        </div>
        <div className="overall">
          <span>API readiness</span>
          <StatusPill state={health.data.status} />
        </div>
      </section>

      <section className="metrics" aria-label="Inventory totals">
        <article>
          <span>Sources</span>
          <strong>{sources.data.length}</strong>
        </article>
        <article>
          <span>Collection runs</span>
          <strong>{collectionRuns.data.length}</strong>
        </article>
        <article>
          <span>Raw artifacts</span>
          <strong>{artifacts.data.length}</strong>
        </article>
      </section>

      <section className="panel">
        <div className="panelHeader">
          <h2>Service Readiness</h2>
          {health.error ? <span className="errorText">{health.error}</span> : null}
        </div>
        <div className="checkGrid">
          {Object.entries(checks).map(([name, check]) => (
            <article className="check" key={name}>
              <span>{name}</span>
              <StatusPill state={check.status} />
              {check.detail ? <small>{check.detail}</small> : null}
            </article>
          ))}
          {Object.keys(checks).length === 0 ? <EmptyState label="No readiness details returned." /> : null}
        </div>
      </section>

      <section className="panel">
        <div className="panelHeader">
          <h2>Sources</h2>
          {sources.error ? <span className="errorText">{sources.error}</span> : null}
        </div>
        <div className="table">
          <div className="row head">
            <span>Name</span>
            <span>Type</span>
            <span>Sensitivity</span>
            <span>Permissions</span>
            <span>State</span>
          </div>
          {sources.data.map((source) => (
            <div className="row" key={source.id}>
              <strong>{source.name}</strong>
              <span>{source.source_type}</span>
              <span>{source.sensitivity}</span>
              <span>{source.permissions?.length ?? 0}</span>
              <StatusPill state={source.enabled ? "enabled" : "disabled"} />
            </div>
          ))}
        </div>
        {sources.data.length === 0 ? <EmptyState label="No sources registered yet." /> : null}
      </section>

      <section className="split">
        <section className="panel">
          <div className="panelHeader">
            <h2>Recent Collection Runs</h2>
            {collectionRuns.error ? <span className="errorText">{collectionRuns.error}</span> : null}
          </div>
          <div className="stack">
            {recentRuns.map((run) => (
              <article className="item" key={run.id}>
                <div>
                  <strong>{compactId(run.id)}</strong>
                  <span>{formatDate(run.created_at)}</span>
                </div>
                <div>
                  <StatusPill state={run.status} />
                  <span>{run.dry_run ? "dry run" : "collection"}</span>
                </div>
              </article>
            ))}
          </div>
          {recentRuns.length === 0 ? <EmptyState label="No collection runs recorded yet." /> : null}
        </section>

        <section className="panel">
          <div className="panelHeader">
            <h2>Recent Raw Artifacts</h2>
            {artifacts.error ? <span className="errorText">{artifacts.error}</span> : null}
          </div>
          <div className="stack">
            {recentArtifacts.map((artifact) => (
              <article className="item" key={artifact.id}>
                <div>
                  <strong>{compactId(artifact.id)}</strong>
                  <span>{formatDate(artifact.created_at)}</span>
                </div>
                <div>
                  <span>{artifact.mime_type ?? "unknown type"}</span>
                  <span>{formatBytes(artifact.size_bytes)}</span>
                </div>
              </article>
            ))}
          </div>
          {recentArtifacts.length === 0 ? <EmptyState label="No raw artifacts recorded yet." /> : null}
        </section>
      </section>
    </main>
  );
}
