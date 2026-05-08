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

type NormalizedDocumentRecord = {
  id: string;
  raw_artifact_id: string | null;
  source_id: string | null;
  title: string | null;
  document_type: string;
  language: string | null;
  sensitivity: string;
  created_at: string;
};

type ChunkRecord = {
  id: string;
  document_id: string;
  chunk_index: number;
  embedding_status: string;
  created_at: string;
};

type EvidenceItemRecord = {
  id: string;
  source_id: string | null;
  document_id: string | null;
  chunk_id: string | null;
  evidence_type: string;
  statement: string;
  confidence: number | null;
  created_at: string;
};

type ClaimRecord = {
  id: string;
  claim_text: string;
  claim_type: string;
  status: string;
  confidence: number | null;
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

function excerpt(value: string, maxLength = 110): string {
  if (value.length <= maxLength) {
    return value;
  }
  return `${value.slice(0, maxLength - 3)}...`;
}

function StatusPill({ state }: { state: string }) {
  return <span className="pill" data-state={state}>{state}</span>;
}

function EmptyState({ label }: { label: string }) {
  return <p className="empty">{label}</p>;
}

export default async function Home() {
  const [health, sources, collectionRuns, artifacts, documents, chunks, evidenceItems, claims] = await Promise.all([
    getJson<HealthResponse>("/health/ready", { status: "error" }),
    getJson<SourceRecord[]>("/sources", []),
    getJson<CollectionRunRecord[]>("/collection-runs", []),
    getJson<RawArtifactRecord[]>("/artifacts", []),
    getJson<NormalizedDocumentRecord[]>("/evidence/documents", []),
    getJson<ChunkRecord[]>("/evidence/chunks", []),
    getJson<EvidenceItemRecord[]>("/evidence/items", []),
    getJson<ClaimRecord[]>("/evidence/claims", [])
  ]);

  const checks = health.data.checks ?? {};
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentArtifacts = artifacts.data.slice(0, 6);
  const recentDocuments = documents.data.slice(0, 5);
  const recentChunks = chunks.data.slice(0, 5);
  const recentEvidence = evidenceItems.data.slice(0, 5);
  const recentClaims = claims.data.slice(0, 5);

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
        <article>
          <span>Evidence items</span>
          <strong>{evidenceItems.data.length}</strong>
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

      <section className="panel">
        <div className="panelHeader">
          <h2>Evidence Explorer</h2>
          {[documents.error, chunks.error, evidenceItems.error, claims.error].filter(Boolean).length > 0 ? (
            <span className="errorText">Some evidence endpoints returned errors.</span>
          ) : null}
        </div>
        <section className="metrics compact" aria-label="Evidence totals">
          <article>
            <span>Documents</span>
            <strong>{documents.data.length}</strong>
          </article>
          <article>
            <span>Chunks</span>
            <strong>{chunks.data.length}</strong>
          </article>
          <article>
            <span>Evidence items</span>
            <strong>{evidenceItems.data.length}</strong>
          </article>
          <article>
            <span>Claims</span>
            <strong>{claims.data.length}</strong>
          </article>
        </section>
        <section className="quad">
          <div>
            <div className="subHeader">
              <h3>Documents</h3>
              {documents.error ? <span className="errorText">{documents.error}</span> : null}
            </div>
            <div className="stack">
              {recentDocuments.map((document) => (
                <article className="item evidenceItem" key={document.id}>
                  <div>
                    <strong>{document.title ?? compactId(document.id)}</strong>
                    <span>{document.document_type} · {document.sensitivity}</span>
                  </div>
                  <div>
                    <span>{formatDate(document.created_at)}</span>
                    <span>source {compactId(document.source_id)}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentDocuments.length === 0 ? <EmptyState label="No normalized documents recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Chunks</h3>
              {chunks.error ? <span className="errorText">{chunks.error}</span> : null}
            </div>
            <div className="stack">
              {recentChunks.map((chunk) => (
                <article className="item evidenceItem" key={chunk.id}>
                  <div>
                    <strong>{compactId(chunk.id)}</strong>
                    <span>document {compactId(chunk.document_id)}</span>
                  </div>
                  <div>
                    <StatusPill state={chunk.embedding_status} />
                    <span>index {chunk.chunk_index}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentChunks.length === 0 ? <EmptyState label="No chunks generated yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Evidence Items</h3>
              {evidenceItems.error ? <span className="errorText">{evidenceItems.error}</span> : null}
            </div>
            <div className="stack">
              {recentEvidence.map((item) => (
                <article className="item evidenceItem" key={item.id}>
                  <div>
                    <strong>{item.evidence_type}</strong>
                    <span>{excerpt(item.statement)}</span>
                  </div>
                  <div>
                    <span>{item.confidence === null ? "unscored" : `${item.confidence}%`}</span>
                    <span>chunk {compactId(item.chunk_id)}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentEvidence.length === 0 ? <EmptyState label="No evidence items recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Claims</h3>
              {claims.error ? <span className="errorText">{claims.error}</span> : null}
            </div>
            <div className="stack">
              {recentClaims.map((claim) => (
                <article className="item evidenceItem" key={claim.id}>
                  <div>
                    <strong>{claim.claim_type}</strong>
                    <span>{excerpt(claim.claim_text)}</span>
                  </div>
                  <div>
                    <StatusPill state={claim.status} />
                    <span>{claim.confidence === null ? "unscored" : `${claim.confidence}%`}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentClaims.length === 0 ? <EmptyState label="No claims recorded yet." /> : null}
          </div>
        </section>
      </section>
    </main>
  );
}
