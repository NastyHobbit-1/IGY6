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

type VectorCollectionStatus = {
  collection_name: string;
  exists: boolean;
};

type GraphSchemaStatus = {
  constraints: Array<Record<string, unknown>>;
};

type PatternRecord = {
  id: string;
  pattern_type: string;
  status: string;
  summary: string;
  confidence: number | null;
  created_at: string;
};

type HypothesisRecord = {
  id: string;
  hypothesis_text: string;
  status: string;
  confidence: number | null;
  created_at: string;
};

type PredictionRecord = {
  id: string;
  prediction_text: string;
  expected_result: string;
  status: string;
  confidence: number | null;
  created_at: string;
};

type RecommendationRecord = {
  id: string;
  recommendation_text: string;
  risk_level: string;
  approval_required: boolean;
  status: string;
  confidence: number | null;
  created_at: string;
};

type WorkItemRecord = {
  id: string;
  work_type: string;
  status: string;
  requested_by_actor_id: string;
  error_message: string | null;
  created_at: string;
};

type ApprovalRecord = {
  id: string;
  request_type: string;
  status: string;
  requested_by_actor_id: string;
  decided_by_actor_id: string | null;
  decision_reason: string | null;
  created_at: string;
};

type FeedbackRecord = {
  id: string;
  target_type: string;
  target_id: string;
  label: string;
  actor_id: string;
  note: string | null;
  created_at: string;
};

type OutcomeRecord = {
  id: string;
  target_type: string;
  target_id: string;
  outcome_status: string;
  summary: string | null;
  created_at: string;
};

type ReportRecord = {
  id: string;
  title: string;
  report_type: string;
  status: string;
  requested_by_actor_id: string;
  created_at: string;
};

type AuditEventRecord = {
  id: number;
  actor_id: string;
  event_type: string;
  decision: string | null;
  resource_type: string | null;
  resource_id: string | null;
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

function ChatRetrievalPreview() {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";

  const script = `
(() => {
  const form = document.querySelector("[data-chat-preview-form]");
  const message = document.querySelector("[data-chat-preview-message]");
  const limit = document.querySelector("[data-chat-preview-limit]");
  const status = document.querySelector("[data-chat-preview-status]");
  const results = document.querySelector("[data-chat-preview-results]");
  const apiBaseUrl = form?.getAttribute("data-api-base-url");

  if (!form || !message || !limit || !status || !results || !apiBaseUrl) {
    return;
  }

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    status.textContent = "Retrieving context";
    results.replaceChildren();

    try {
      const response = await fetch(apiBaseUrl + "/chat/retrieval-preview", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          message: message.value,
          limit: Number(limit.value || 5)
        })
      });

      if (!response.ok) {
        status.textContent = "Error: " + response.status + " " + response.statusText;
        return;
      }

      const payload = await response.json();
      const hits = payload.retrieval_context?.hits ?? [];
      status.textContent = "answer_status: " + payload.answer_status + " | hits: " + hits.length;

      if (hits.length === 0) {
        const empty = document.createElement("p");
        empty.className = "empty";
        empty.textContent = "No retrieval context returned.";
        results.appendChild(empty);
        return;
      }

      for (const hit of hits) {
        const item = document.createElement("article");
        item.className = "item evidenceItem";

        const left = document.createElement("div");
        const title = document.createElement("strong");
        title.textContent = hit.document?.title || hit.chunk?.id || "retrieval hit";
        const detail = document.createElement("span");
        detail.textContent = "score " + hit.score + " | chunk " + (hit.chunk?.id || "unknown");
        left.append(title, detail);

        const right = document.createElement("div");
        const evidence = document.createElement("span");
        evidence.textContent = (hit.evidence_items?.length ?? 0) + " evidence items";
        const source = document.createElement("span");
        source.textContent = "source " + (hit.source?.name || hit.source?.id || "none");
        right.append(evidence, source);

        item.append(left, right);
        results.appendChild(item);
      }
    } catch (error) {
      status.textContent = "Error: " + (error instanceof Error ? error.message : "Unknown error");
    }
  });
})();
`;

  return (
    <section className="panel">
      <div className="panelHeader">
        <h2>Chat Retrieval Preview</h2>
        <span className="statusText" data-chat-preview-status>answer_status: not_generated</span>
      </div>
      <form className="previewForm" data-chat-preview-form data-api-base-url={browserApiBaseUrl}>
        <label>
          <span>Message</span>
          <textarea data-chat-preview-message name="message" rows={3} defaultValue="What does the system know?" />
        </label>
        <label>
          <span>Limit</span>
          <input data-chat-preview-limit name="limit" type="number" min="1" max="50" defaultValue="5" />
        </label>
        <button type="submit">Preview Retrieval Context</button>
      </form>
      <div className="previewNote">
        This preview returns retrieval context only. It does not generate an answer, persist a conversation, call a model, or trigger an action.
      </div>
      <div className="stack previewResults" data-chat-preview-results />
      <script dangerouslySetInnerHTML={{ __html: script }} />
    </section>
  );
}

export default async function Home() {
  const [
    health,
    sources,
    collectionRuns,
    artifacts,
    documents,
    chunks,
    evidenceItems,
    claims,
    vectorCollection,
    graphSchema,
    patterns,
    hypotheses,
    predictions,
    recommendations,
    workItems,
    approvals,
    feedback,
    outcomes,
    reports,
    auditEvents
  ] = await Promise.all([
    getJson<HealthResponse>("/health/ready", { status: "error" }),
    getJson<SourceRecord[]>("/sources", []),
    getJson<CollectionRunRecord[]>("/collection-runs", []),
    getJson<RawArtifactRecord[]>("/artifacts", []),
    getJson<NormalizedDocumentRecord[]>("/evidence/documents", []),
    getJson<ChunkRecord[]>("/evidence/chunks", []),
    getJson<EvidenceItemRecord[]>("/evidence/items", []),
    getJson<ClaimRecord[]>("/evidence/claims", []),
    getJson<VectorCollectionStatus>("/memory/vector/chunks", { collection_name: "unknown", exists: false }),
    getJson<GraphSchemaStatus>("/memory/graph/schema", { constraints: [] }),
    getJson<PatternRecord[]>("/analysis/patterns", []),
    getJson<HypothesisRecord[]>("/analysis/hypotheses", []),
    getJson<PredictionRecord[]>("/analysis/predictions", []),
    getJson<RecommendationRecord[]>("/analysis/recommendations", []),
    getJson<WorkItemRecord[]>("/work-items", []),
    getJson<ApprovalRecord[]>("/approvals", []),
    getJson<FeedbackRecord[]>("/feedback", []),
    getJson<OutcomeRecord[]>("/outcomes", []),
    getJson<ReportRecord[]>("/reports", []),
    getJson<AuditEventRecord[]>("/audit-events", [])
  ]);

  const checks = health.data.checks ?? {};
  const recentRuns = collectionRuns.data.slice(0, 6);
  const recentArtifacts = artifacts.data.slice(0, 6);
  const recentDocuments = documents.data.slice(0, 5);
  const recentChunks = chunks.data.slice(0, 5);
  const recentEvidence = evidenceItems.data.slice(0, 5);
  const recentClaims = claims.data.slice(0, 5);
  const recentPatterns = patterns.data.slice(0, 4);
  const recentHypotheses = hypotheses.data.slice(0, 4);
  const recentPredictions = predictions.data.slice(0, 4);
  const recentRecommendations = recommendations.data.slice(0, 4);
  const recentWorkItems = workItems.data.slice(0, 4);
  const recentApprovals = approvals.data.slice(0, 4);
  const recentFeedback = feedback.data.slice(0, 4);
  const recentOutcomes = outcomes.data.slice(0, 4);
  const recentReports = reports.data.slice(0, 4);
  const recentAuditEvents = auditEvents.data.slice(0, 4);

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

      <section className="panel">
        <div className="panelHeader">
          <h2>Memory And Analysis</h2>
          {[vectorCollection.error, graphSchema.error, patterns.error, hypotheses.error, predictions.error, recommendations.error].filter(Boolean).length > 0 ? (
            <span className="errorText">Some memory or analysis endpoints returned errors.</span>
          ) : null}
        </div>
        <section className="metrics compact" aria-label="Memory and analysis totals">
          <article>
            <span>Vector collection</span>
            <strong>{vectorCollection.data.exists ? "Ready" : "Missing"}</strong>
          </article>
          <article>
            <span>Graph constraints</span>
            <strong>{graphSchema.data.constraints.length}</strong>
          </article>
          <article>
            <span>Patterns</span>
            <strong>{patterns.data.length}</strong>
          </article>
          <article>
            <span>Recommendations</span>
            <strong>{recommendations.data.length}</strong>
          </article>
        </section>
        <section className="split">
          <div className="memoryStatus">
            <div className="subHeader">
              <h3>Vector Memory</h3>
              {vectorCollection.error ? <span className="errorText">{vectorCollection.error}</span> : null}
            </div>
            <article className="item evidenceItem">
              <div>
                <strong>{vectorCollection.data.collection_name}</strong>
                <span>Configured chunk collection</span>
              </div>
              <div>
                <StatusPill state={vectorCollection.data.exists ? "enabled" : "missing"} />
              </div>
            </article>
          </div>
          <div className="memoryStatus">
            <div className="subHeader">
              <h3>Graph Memory</h3>
              {graphSchema.error ? <span className="errorText">{graphSchema.error}</span> : null}
            </div>
            <article className="item evidenceItem">
              <div>
                <strong>{graphSchema.data.constraints.length} constraints</strong>
                <span>Schema inspection only</span>
              </div>
              <div>
                <StatusPill state={graphSchema.error ? "error" : "ok"} />
              </div>
            </article>
          </div>
        </section>
        <section className="quad analysisGrid">
          <div>
            <div className="subHeader">
              <h3>Patterns</h3>
              {patterns.error ? <span className="errorText">{patterns.error}</span> : null}
            </div>
            <div className="stack">
              {recentPatterns.map((pattern) => (
                <article className="item evidenceItem" key={pattern.id}>
                  <div>
                    <strong>{pattern.pattern_type}</strong>
                    <span>{excerpt(pattern.summary)}</span>
                  </div>
                  <div>
                    <StatusPill state={pattern.status} />
                    <span>{pattern.confidence === null ? "unscored" : `${pattern.confidence}%`}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentPatterns.length === 0 ? <EmptyState label="No patterns recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Hypotheses</h3>
              {hypotheses.error ? <span className="errorText">{hypotheses.error}</span> : null}
            </div>
            <div className="stack">
              {recentHypotheses.map((hypothesis) => (
                <article className="item evidenceItem" key={hypothesis.id}>
                  <div>
                    <strong>{compactId(hypothesis.id)}</strong>
                    <span>{excerpt(hypothesis.hypothesis_text)}</span>
                  </div>
                  <div>
                    <StatusPill state={hypothesis.status} />
                    <span>{hypothesis.confidence === null ? "unscored" : `${hypothesis.confidence}%`}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentHypotheses.length === 0 ? <EmptyState label="No hypotheses recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Predictions</h3>
              {predictions.error ? <span className="errorText">{predictions.error}</span> : null}
            </div>
            <div className="stack">
              {recentPredictions.map((prediction) => (
                <article className="item evidenceItem" key={prediction.id}>
                  <div>
                    <strong>{excerpt(prediction.prediction_text, 80)}</strong>
                    <span>{excerpt(prediction.expected_result, 90)}</span>
                  </div>
                  <div>
                    <StatusPill state={prediction.status} />
                    <span>{prediction.confidence === null ? "unscored" : `${prediction.confidence}%`}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentPredictions.length === 0 ? <EmptyState label="No predictions recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Recommendations</h3>
              {recommendations.error ? <span className="errorText">{recommendations.error}</span> : null}
            </div>
            <div className="stack">
              {recentRecommendations.map((recommendation) => (
                <article className="item evidenceItem" key={recommendation.id}>
                  <div>
                    <strong>{recommendation.risk_level}</strong>
                    <span>{excerpt(recommendation.recommendation_text)}</span>
                  </div>
                  <div>
                    <StatusPill state={recommendation.status} />
                    <span>{recommendation.approval_required ? "approval" : "no approval"}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentRecommendations.length === 0 ? <EmptyState label="No recommendations recorded yet." /> : null}
          </div>
        </section>
      </section>

      <section className="panel">
        <div className="panelHeader">
          <h2>Review And Operations</h2>
          {[workItems.error, approvals.error, feedback.error, outcomes.error, reports.error, auditEvents.error].filter(Boolean).length > 0 ? (
            <span className="errorText">Some review or operations endpoints returned errors.</span>
          ) : null}
        </div>
        <section className="metrics compact" aria-label="Review and operations totals">
          <article>
            <span>Work items</span>
            <strong>{workItems.data.length}</strong>
          </article>
          <article>
            <span>Approvals</span>
            <strong>{approvals.data.length}</strong>
          </article>
          <article>
            <span>Feedback</span>
            <strong>{feedback.data.length}</strong>
          </article>
          <article>
            <span>Audit events</span>
            <strong>{auditEvents.data.length}</strong>
          </article>
        </section>
        <section className="quad analysisGrid">
          <div>
            <div className="subHeader">
              <h3>Work Items</h3>
              {workItems.error ? <span className="errorText">{workItems.error}</span> : null}
            </div>
            <div className="stack">
              {recentWorkItems.map((workItem) => (
                <article className="item evidenceItem" key={workItem.id}>
                  <div>
                    <strong>{workItem.work_type}</strong>
                    <span>{workItem.error_message ?? `requested by ${workItem.requested_by_actor_id}`}</span>
                  </div>
                  <div>
                    <StatusPill state={workItem.status} />
                    <span>{formatDate(workItem.created_at)}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentWorkItems.length === 0 ? <EmptyState label="No work items recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Approvals</h3>
              {approvals.error ? <span className="errorText">{approvals.error}</span> : null}
            </div>
            <div className="stack">
              {recentApprovals.map((approval) => (
                <article className="item evidenceItem" key={approval.id}>
                  <div>
                    <strong>{approval.request_type}</strong>
                    <span>{approval.decision_reason ?? `requested by ${approval.requested_by_actor_id}`}</span>
                  </div>
                  <div>
                    <StatusPill state={approval.status} />
                    <span>{approval.decided_by_actor_id ?? "undecided"}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentApprovals.length === 0 ? <EmptyState label="No approvals recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Feedback</h3>
              {feedback.error ? <span className="errorText">{feedback.error}</span> : null}
            </div>
            <div className="stack">
              {recentFeedback.map((event) => (
                <article className="item evidenceItem" key={event.id}>
                  <div>
                    <strong>{event.label}</strong>
                    <span>{event.note ?? `${event.target_type} ${compactId(event.target_id)}`}</span>
                  </div>
                  <div>
                    <span>{event.actor_id}</span>
                    <span>{formatDate(event.created_at)}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentFeedback.length === 0 ? <EmptyState label="No feedback recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Outcomes</h3>
              {outcomes.error ? <span className="errorText">{outcomes.error}</span> : null}
            </div>
            <div className="stack">
              {recentOutcomes.map((outcome) => (
                <article className="item evidenceItem" key={outcome.id}>
                  <div>
                    <strong>{outcome.target_type}</strong>
                    <span>{outcome.summary ?? compactId(outcome.target_id)}</span>
                  </div>
                  <div>
                    <StatusPill state={outcome.outcome_status} />
                    <span>{formatDate(outcome.created_at)}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentOutcomes.length === 0 ? <EmptyState label="No outcomes recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Reports</h3>
              {reports.error ? <span className="errorText">{reports.error}</span> : null}
            </div>
            <div className="stack">
              {recentReports.map((report) => (
                <article className="item evidenceItem" key={report.id}>
                  <div>
                    <strong>{report.title}</strong>
                    <span>{report.report_type}</span>
                  </div>
                  <div>
                    <StatusPill state={report.status} />
                    <span>{report.requested_by_actor_id}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentReports.length === 0 ? <EmptyState label="No reports recorded yet." /> : null}
          </div>

          <div>
            <div className="subHeader">
              <h3>Audit Events</h3>
              {auditEvents.error ? <span className="errorText">{auditEvents.error}</span> : null}
            </div>
            <div className="stack">
              {recentAuditEvents.map((event) => (
                <article className="item evidenceItem" key={event.id}>
                  <div>
                    <strong>{event.event_type}</strong>
                    <span>{event.resource_type ?? "resource"} {compactId(event.resource_id)}</span>
                  </div>
                  <div>
                    <StatusPill state={event.decision ?? "recorded"} />
                    <span>{event.actor_id}</span>
                  </div>
                </article>
              ))}
            </div>
            {recentAuditEvents.length === 0 ? <EmptyState label="No audit events recorded yet." /> : null}
          </div>
        </section>
      </section>

      <ChatRetrievalPreview />
    </main>
  );
}
