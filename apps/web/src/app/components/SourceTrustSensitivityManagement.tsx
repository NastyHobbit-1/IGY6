import type { SourceRecord, CollectionRunRecord, NormalizedDocumentRecord, EvidenceItemRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function SourceTrustSensitivityManagement({
  sources,
  collectionRuns,
  documents,
  evidenceItems
}: {
  sources: ApiResult<SourceRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  documents: ApiResult<NormalizedDocumentRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
}) {
  const reviewableSources = sources.data.slice(0, 12);
  const sourceReviewRows = reviewableSources.map((source) => {
    const runCount = collectionRuns.data.filter((run) => run.source_id === source.id).length;
    const documentCount = documents.data.filter((document) => document.source_id === source.id).length;
    const evidenceCount = evidenceItems.data.filter((item) => item.source_id === source.id).length;
    return {
      id: source.id,
      name: source.name,
      source_type: source.source_type,
      sensitivity: source.sensitivity,
      trust_level: source.trust_level,
      enabled: source.enabled,
      updated_at: source.updated_at ?? null,
      run_count: runCount,
      document_count: documentCount,
      evidence_count: evidenceCount
    };
  });
  const sourceReviewJson = JSON.stringify(sourceReviewRows).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-source-review-management]");
  if (!root) return;
  const sources = JSON.parse(root.querySelector("[data-source-review-json]")?.textContent || "[]");
  const form = root.querySelector("[data-source-review-form]");
  const result = root.querySelector("[data-source-review-result]");
  const sourceSelect = root.querySelector("[name='source_review_source']");
  const stateSelect = root.querySelector("[name='source_review_state']");
  const sensitivitySelect = root.querySelector("[name='source_review_sensitivity']");
  const enabledInput = root.querySelector("[name='source_review_enabled']");
  const sourceSummary = root.querySelector("[data-source-review-selected]");
  const stateToTrust = (state) => state === "review_needed" ? "review_needed" : state;
  const selectedSource = () => sources.find((source) => source.id === sourceSelect?.value) || null;
  const stateForSource = (source) => {
    const current = source?.trust_level || "review_needed";
    return ["trusted", "noisy", "sensitive", "disabled", "review_needed"].includes(current) ? current : "review_needed";
  };
  const show = (state, message, payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = state;
    const body = document.createElement("span");
    body.textContent = message;
    result.append(title, body);
    if (payload) {
      const details = document.createElement("dl");
      details.setAttribute("data-source-review-status", "");
      [["source", payload.id], ["trust", payload.trust_level], ["sensitivity", payload.sensitivity], ["enabled", String(payload.enabled)]].forEach(([label, value]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = value || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const refreshSelected = () => {
    const source = selectedSource();
    if (!source) {
      if (sourceSummary) sourceSummary.textContent = "No source is available for review.";
      return;
    }
    if (stateSelect) stateSelect.value = stateForSource(source);
    if (sensitivitySelect) sensitivitySelect.value = source.sensitivity || "internal";
    if (enabledInput) enabledInput.checked = Boolean(source.enabled);
    if (sourceSummary) {
      sourceSummary.textContent = source.name + " has " + source.evidence_count + " evidence item(s), " + source.document_count + " document(s), and " + source.run_count + " collection run(s). Existing evidence stays visible after review updates.";
    }
  };
  stateSelect?.addEventListener("change", () => {
    if (enabledInput && stateSelect.value === "disabled") enabledInput.checked = false;
    if (sensitivitySelect && stateSelect.value === "sensitive") sensitivitySelect.value = "sensitive";
  });
  sourceSelect?.addEventListener("change", refreshSelected);
  refreshSelected();
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const source = selectedSource();
    if (!source) {
      show("No source selected", "Register a source before reviewing trust and sensitivity.");
      return;
    }
    const selectedState = stateSelect?.value || "review_needed";
    const enabled = selectedState === "disabled" ? false : Boolean(enabledInput?.checked);
    try {
      const response = await fetch("/api/sources/" + encodeURIComponent(source.id) + "/review-state", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          trust_level: stateToTrust(selectedState),
          sensitivity: sensitivitySelect?.value || "internal",
          enabled,
          review_note: root.querySelector("[name='source_review_note']")?.value?.trim() || null,
          actor_id: "local-owner"
        })
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
      show("Source review saved", "IGY6 updated the source record and audit trail. Reload to see refreshed source lists. Existing evidence was not hidden or deleted.", payload);
    } catch (error) {
      show("Source review failed", String(error));
    }
  });
})();
`;

  return (
    <section className="guidedManualText sourceReviewManagement" data-source-review-management>
      <div className="guidedManualNotice">
        <strong>Source trust and sensitivity</strong>
        <span>Review source state for future use. This does not delete sources, rewrite historical evidence, or silently hide evidence from Results.</span>
      </div>
      {sources.error ? <p className="errorText">Source state could not be loaded: {sources.error}</p> : null}
      <section className="stack" aria-label="Source review summary">
        {sourceReviewRows.slice(0, 6).map((source) => (
          <article className="item evidenceItem" key={source.id} data-source-review-item>
            <div>
              <strong>{source.name}</strong>
              <span>{source.source_type} · {source.sensitivity}</span>
              <span>{source.evidence_count} evidence item(s), {source.document_count} document(s), {source.run_count} collection run(s)</span>
            </div>
            <div>
              <StatusPill state={source.enabled ? "enabled" : "disabled"} />
              <StatusPill state={source.trust_level || "review_needed"} />
            </div>
          </article>
        ))}
      </section>
      {sourceReviewRows.length === 0 ? <EmptyState label="No sources are available for trust or sensitivity review yet." /> : null}
      <form className="guidedManualForm" data-source-review-form>
        <label>
          <span>Source</span>
          <select name="source_review_source" disabled={sourceReviewRows.length === 0}>
            {sourceReviewRows.map((source) => (
              <option key={source.id} value={source.id}>{source.name} · {source.source_type}</option>
            ))}
          </select>
        </label>
        <p className="actionHint" data-source-review-selected>
          {sourceReviewRows.length > 0 ? "Choose a source to review linked evidence counts." : "Register a source before reviewing trust state."}
        </p>
        <label>
          <span>Trust state</span>
          <select name="source_review_state" defaultValue="review_needed" disabled={sourceReviewRows.length === 0}>
            <option value="trusted">trusted</option>
            <option value="noisy">noisy</option>
            <option value="sensitive">sensitive</option>
            <option value="disabled">disabled</option>
            <option value="review_needed">review-needed</option>
          </select>
        </label>
        <label>
          <span>Sensitivity label</span>
          <select name="source_review_sensitivity" defaultValue="internal" disabled={sourceReviewRows.length === 0}>
            <option value="public">public</option>
            <option value="internal">internal</option>
            <option value="sensitive">sensitive</option>
            <option value="secret">secret</option>
          </select>
        </label>
        <label className="checkLine">
          <input name="source_review_enabled" type="checkbox" defaultChecked disabled={sourceReviewRows.length === 0} />
          Enabled for future collection workflows
        </label>
        <label>
          <span>Review note</span>
          <textarea name="source_review_note" rows={2} placeholder="Optional note explaining the review decision." disabled={sourceReviewRows.length === 0} />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={sourceReviewRows.length === 0}>Save source review</button>
          <span>Updates source metadata and audit records only; retrieval ranking and policy enforcement are not changed here.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-source-review-result>
        <strong>{sourceReviewRows.length > 0 ? "Ready for review" : "No source to review"}</strong>
        <span>{sourceReviewRows.length > 0 ? "Choose a source and save a real state update." : "Create a source first in Add Data."}</span>
      </div>
      <DomJsonScript marker="data-source-review-json" json={sourceReviewJson} />
      <ClientScript script={script} />
    </section>
  );
}

