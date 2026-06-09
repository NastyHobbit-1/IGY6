import type { EvidenceItemRecord, ApiResult } from "./types";
import { excerpt, evidenceReviewState, evidenceReviewNote } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function EvidenceCorrectionSupersessionWorkflow({
  evidenceItems
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
}) {
  const reviewableEvidence = evidenceItems.data.slice(0, 12);
  const evidenceRows = reviewableEvidence.map((item) => ({
    id: item.id,
    evidence_type: item.evidence_type,
    statement_preview: excerpt(item.statement, 96),
    review_state: evidenceReviewState(item),
    correction_note: evidenceReviewNote(item),
    source_id: item.source_id,
    document_id: item.document_id,
    chunk_id: item.chunk_id
  }));
  const evidenceRowsJson = JSON.stringify(evidenceRows).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-evidence-correction-workflow]");
  if (!root) return;
  const evidence = JSON.parse(root.querySelector("[data-evidence-correction-json]")?.textContent || "[]");
  const form = root.querySelector("[data-evidence-correction-form]");
  const evidenceSelect = root.querySelector("[name='evidence_correction_target']");
  const stateSelect = root.querySelector("[name='evidence_correction_state']");
  const supersedingSelect = root.querySelector("[name='evidence_superseding_id']");
  const result = root.querySelector("[data-evidence-correction-result]");
  const selectedEvidence = () => evidence.find((item) => item.id === evidenceSelect?.value) || null;
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
      details.setAttribute("data-evidence-correction-status", "");
      const reviewState = payload.metadata_json?.review_state || {};
      [["evidence", payload.id], ["state", reviewState.state], ["supersedes", reviewState.superseding_evidence_item_id || "not linked"], ["history", "original evidence preserved"]].forEach(([label, value]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = value || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const refreshSupersedingOptions = () => {
    const selected = selectedEvidence();
    if (!supersedingSelect || !selected) return;
    Array.from(supersedingSelect.options).forEach((option) => {
      option.disabled = option.value === selected.id;
    });
    if (supersedingSelect.value === selected.id) supersedingSelect.value = "";
  };
  evidenceSelect?.addEventListener("change", refreshSupersedingOptions);
  refreshSupersedingOptions();
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const selected = selectedEvidence();
    if (!selected) {
      show("No evidence selected", "Process text into evidence before recording correction state.");
      return;
    }
    try {
      const response = await fetch("/api/evidence/items/" + encodeURIComponent(selected.id) + "/review-state", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({
          review_state: stateSelect?.value || "needs_correction",
          correction_note: root.querySelector("[name='evidence_correction_note']")?.value?.trim() || null,
          superseding_evidence_item_id: supersedingSelect?.value || null,
          actor_id: "local-owner"
        })
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
      show("Evidence review saved", "IGY6 recorded review metadata and audit history. Original evidence, source, document, chunk, and artifact records were not deleted or rewritten.", payload);
    } catch (error) {
      show("Evidence review failed", String(error));
    }
  });
})();
`;

  return (
    <section className="guidedManualText evidenceCorrectionWorkflow" data-evidence-correction-workflow>
      <div className="guidedManualNotice">
        <strong>Evidence correction and supersession</strong>
        <span>Mark evidence review state without deleting, rewriting, or hiding the original evidence. Retrieval ranking and filtering are not changed here.</span>
      </div>
      {evidenceItems.error ? <p className="errorText">Evidence items could not be loaded: {evidenceItems.error}</p> : null}
      <section className="stack" aria-label="Evidence correction summary">
        {evidenceRows.slice(0, 6).map((item) => (
          <article className="item evidenceItem" key={item.id} data-evidence-correction-item>
            <div>
              <strong>{item.evidence_type}</strong>
              <span>{item.statement_preview}</span>
              <span>{item.correction_note ?? "No correction note recorded."}</span>
            </div>
            <div>
              <StatusPill state={item.review_state} />
              <span>{item.chunk_id ? "chunk-linked" : item.document_id ? "document-linked" : item.source_id ? "source-linked" : "lineage missing"}</span>
            </div>
          </article>
        ))}
      </section>
      {evidenceRows.length === 0 ? <EmptyState label="No evidence items are available for correction review yet." /> : null}
      <form className="guidedManualForm" data-evidence-correction-form>
        <label>
          <span>Evidence item</span>
          <select name="evidence_correction_target" disabled={evidenceRows.length === 0}>
            {evidenceRows.map((item) => (
              <option key={item.id} value={item.id}>{item.evidence_type} · {item.id}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Review state</span>
          <select name="evidence_correction_state" defaultValue="needs_correction" disabled={evidenceRows.length === 0}>
            <option value="needs_correction">needs correction</option>
            <option value="corrected">corrected</option>
            <option value="superseded">superseded</option>
            <option value="disputed">disputed</option>
            <option value="verified">verified</option>
          </select>
        </label>
        <label>
          <span>Superseding evidence</span>
          <select name="evidence_superseding_id" defaultValue="" disabled={evidenceRows.length < 2}>
            <option value="">No superseding evidence link</option>
            {evidenceRows.map((item) => (
              <option key={item.id} value={item.id}>{item.evidence_type} · {item.id}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Correction note</span>
          <textarea name="evidence_correction_note" rows={2} placeholder="Short note explaining the correction, dispute, or supersession." disabled={evidenceRows.length === 0} />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={evidenceRows.length === 0}>Save evidence review</button>
          <span>Records additive review metadata only. Existing source, artifact, document, chunk, and evidence history stays visible.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-evidence-correction-result>
        <strong>{evidenceRows.length > 0 ? "Ready for evidence review" : "No evidence to review"}</strong>
        <span>{evidenceRows.length > 0 ? "Choose an evidence item and save a real correction state." : "Add and process supported text before reviewing evidence correction state."}</span>
      </div>
      <DomJsonScript marker="data-evidence-correction-json" json={evidenceRowsJson} />
      <ClientScript script={script} />
    </section>
  );
}

