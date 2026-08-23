import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { TermHelp } from "./ui/TermHelp";
import { HelpHeading } from "./ui/HelpHeading";

export function ChatRetrievalPreview() {
  const browserApiBaseUrl = "/api";

  const script = `
(() => {
  const form = document.querySelector("[data-chat-preview-form]");
  const message = document.querySelector("[data-chat-preview-message]");
  const limit = document.querySelector("[data-chat-preview-limit]");
  const status = document.querySelector("[data-chat-preview-status]");
  const results = document.querySelector("[data-chat-preview-results]");
  const saveButton = document.querySelector("[data-chat-save-answer]");
  const saveStatus = document.querySelector("[data-chat-save-answer-status]");
  const apiBaseUrl = form?.getAttribute("data-api-base-url");
  let lastPayload = null;
  let lastQuestion = "";
  let lastRequest = null;

  if (!form || !message || !limit || !status || !results || !apiBaseUrl) {
    return;
  }
  if (form.getAttribute("data-preview-wired") === "true") return;
  form.setAttribute("data-preview-wired", "true");

  const shortId = (value) => {
    if (!value || typeof value !== "string") return "unknown";
    return value.length > 12 ? value.slice(0, 8) + "..." : value;
  };

  const formatScore = (value) => {
    const score = Number(value);
    return Number.isFinite(score) ? score.toFixed(3) : "unscored";
  };

  const textPreview = (value, fallback) => {
    const text = typeof value === "string" ? value.replace(/\\s+/g, " ").trim() : "";
    if (!text) return fallback;
    return text.length > 280 ? text.slice(0, 277) + "..." : text;
  };

  const retrievalMode = (hit) => {
    return hit.qdrant_payload?.retrieval_mode
      || hit.qdrant_payload?.embedding_method
      || hit.qdrant_payload?.payload?.retrieval_mode
      || "text or vector search";
  };

  const addMeta = (parent, label, value) => {
    const meta = document.createElement("span");
    meta.textContent = label + ": " + value;
    parent.appendChild(meta);
  };

  const uniqueStrings = (values, maxItems) => {
    const seen = new Set();
    const result = [];
    for (const value of values) {
      if (typeof value !== "string") continue;
      const trimmed = value.trim();
      if (!trimmed || seen.has(trimmed)) continue;
      seen.add(trimmed);
      result.push(trimmed);
      if (result.length >= maxItems) break;
    }
    return result;
  };

  const answerStatusFor = (payload, hits) => {
    const status = typeof payload.answer_status === "string" ? payload.answer_status : "";
    if (status) return status;
    return hits.length > 0 ? "evidence_summary" : "insufficient_evidence";
  };

  const statementFromHit = (hit) => {
    const evidenceStatement = Array.isArray(hit.evidence_items)
      ? hit.evidence_items.map((item) => item.statement).find((value) => typeof value === "string" && value.trim())
      : "";
    return textPreview(evidenceStatement || hit.chunk?.text_content, "Retrieved hit has no text preview.");
  };

  const sourceTrailFromHit = (hit) => {
    const sourceLabel = hit.source?.name || shortId(hit.source?.id);
    const documentLabel = hit.document?.title || shortId(hit.document?.id || hit.qdrant_payload?.document_id);
    const chunkLabel = shortId(hit.chunk?.id || hit.qdrant_payload?.chunk_id);
    return "source " + sourceLabel + " > document " + documentLabel + " > chunk " + chunkLabel + " (score " + formatScore(hit.score) + ")";
  };

  const buildGroundedAnswerPacket = (payload, hits) => {
    const evidenceItemIds = uniqueStrings(hits.flatMap((hit) => Array.isArray(hit.evidence_items) ? hit.evidence_items.map((item) => item.id) : []), 50);
    const documentIds = uniqueStrings(hits.map((hit) => hit.document?.id || hit.qdrant_payload?.document_id), 50);
    const chunkIds = uniqueStrings(hits.map((hit) => hit.chunk?.id || hit.qdrant_payload?.chunk_id), 50);
    const sourceIds = uniqueStrings(hits.map((hit) => hit.source?.id), 50);
    const sourceTrails = uniqueStrings(hits.map(sourceTrailFromHit), 20);
    const facts = uniqueStrings(hits.map(statementFromHit), 10);
    const citationLabels = uniqueStrings([
      ...evidenceItemIds.map((id) => "evidence " + shortId(id)),
      ...documentIds.map((id) => "document " + shortId(id)),
      ...chunkIds.map((id) => "chunk " + shortId(id)),
      ...sourceIds.map((id) => "source " + shortId(id))
    ], 30);
    const hitCount = hits.length;
    return {
      answer_status: answerStatusFor(payload, hits),
      answer_text: hitCount > 0
        ? "Deterministic evidence-grounded answer packet from " + hitCount + " retrieved local evidence hit(s). Treat it as a cited review aid, not verified truth."
        : "Insufficient evidence: no matching local chunks or evidence items were retrieved for this question.",
      facts,
      assumptions: [
        "Stored source metadata and evidence records are treated as local records of what was collected.",
        "Retrieval scores are similarity signals, not proof of correctness."
      ],
      inferences: hitCount > 0
        ? ["The available answer is limited to the retrieved local evidence and citation labels shown here."]
        : [],
      uncertainty: [
        "This packet uses deterministic local retrieval context only.",
        "No hosted AI, hidden reasoning, browser scraping, account scraping, or full chat memory was used.",
        "Relevant sources not yet ingested, chunked, or embedded are absent from this packet."
      ],
      missing_information: hitCount > 0
        ? ["Any relevant local source not yet ingested, chunked, and embedded is missing from this answer."]
        : ["No matching local chunks or evidence items were retrieved. Add or process relevant local evidence before drawing a conclusion."],
      evidence_item_ids: evidenceItemIds,
      document_ids: documentIds,
      chunk_ids: chunkIds,
      source_ids: sourceIds,
      safe_labels: citationLabels,
      source_trails: sourceTrails,
      retrieval_count: hitCount,
      retrieval_mode: "retrieval_preview",
      local_model_status: "not_called_retrieval_preview_deterministic",
      local_model_detail: "Local model/provider contribution was not requested by this retrieval-preview path; deterministic fallback is shown."
    };
  };

  const buildAnswerRecordPayload = () => {
    const payload = lastPayload || {};
    const hits = Array.isArray(payload.retrieval_context?.hits) ? payload.retrieval_context.hits : [];
    const packet = buildGroundedAnswerPacket(payload, hits);
    return {
      user_question: lastQuestion,
      answer_status: packet.answer_status,
      answer_text: packet.answer_text,
      facts: packet.facts,
      assumptions: packet.assumptions,
      inferences: packet.inferences,
      uncertainty: packet.uncertainty,
      missing_information: packet.missing_information,
      evidence_item_ids: packet.evidence_item_ids,
      document_ids: packet.document_ids,
      chunk_ids: packet.chunk_ids,
      source_ids: packet.source_ids,
      safe_labels: packet.safe_labels,
      retrieval_mode: packet.retrieval_mode,
      retrieval_count: packet.retrieval_count,
      local_model_status: packet.local_model_status,
      metadata_json: {
        created_from: "results_evidence_grounded_answer_packet",
        raw_evidence_text_stored: false,
        full_chat_memory: false,
        hosted_ai_called: false,
        answer_packet_available: true,
        retrieval_context_available: true
      }
    };
  };

  const renderReviewSummary = (payload, hits) => {
    const summary = document.createElement("article");
    summary.className = "item evidenceItem";
    summary.setAttribute("data-retrieval-review-summary", "");

    const left = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = hits.length > 0 ? "Evidence retrieved" : "No evidence found";
    const detail = document.createElement("span");
    detail.textContent = hits.length > 0
      ? "Evidence-backed review: use the chunks and evidence items below as support."
      : "Insufficient evidence: try a narrower question or add/process more local evidence.";
    left.append(title, detail);

    const right = document.createElement("div");
    addMeta(right, "answer_status", payload.answer_status || "unknown");
    addMeta(right, "hits", String(hits.length));
    addMeta(right, "collection", payload.retrieval_context?.collection_exists === false ? "missing" : "available");
    summary.append(left, right);
    return summary;
  };

  const renderPacketList = (parent, label, values, emptyText) => {
    const section = document.createElement("p");
    section.className = "messageMeta";
    const title = document.createElement("strong");
    title.textContent = label + ": ";
    section.appendChild(title);
    const items = Array.isArray(values) ? values.filter((value) => typeof value === "string" && value.trim()) : [];
    section.append(items.length > 0 ? items.join(" | ") : emptyText);
    parent.appendChild(section);
  };

  const renderAnswerPacket = (payload, hits) => {
    const packet = buildGroundedAnswerPacket(payload, hits);
    const item = document.createElement("article");
    item.className = "item evidenceItem";
    item.setAttribute("data-evidence-grounded-answer-packet", "");

    const body = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = "Evidence-grounded answer packet";
    const detail = document.createElement("span");
    detail.textContent = packet.answer_text;
    body.append(title, detail);

    renderPacketList(body, "Facts", packet.facts, "No facts extracted from retrieved evidence.");
    renderPacketList(body, "Assumptions", packet.assumptions, "No assumptions recorded.");
    renderPacketList(body, "Inferences", packet.inferences, "No inference made without evidence.");
    renderPacketList(body, "Uncertainty", packet.uncertainty, "No uncertainty recorded.");
    renderPacketList(body, "Missing information", packet.missing_information, "No missing information recorded.");
    renderPacketList(body, "Citations", packet.safe_labels, "No citation labels available.");
    renderPacketList(body, "Source trail", packet.source_trails, "No source/document/chunk trail available.");

    const right = document.createElement("div");
    addMeta(right, "answer_status", packet.answer_status);
    addMeta(right, "retrieval_hits", String(packet.retrieval_count));
    addMeta(right, "retrieved evidence", hits.length > 0 ? "shown below" : "none");
    addMeta(right, "packet", "deterministic");
    addMeta(right, "local model", packet.local_model_status);
    addMeta(right, "provider", "not used by retrieval preview");
    addMeta(right, "fallback", "deterministic evidence-only");
    item.append(body, right);

    const llmLine = document.createElement("p");
    llmLine.className = "messageMeta";
    llmLine.textContent = packet.local_model_detail;
    item.appendChild(llmLine);
    return item;
  };

  const renderHit = (hit, index) => {
    const item = document.createElement("article");
    item.className = "item evidenceItem";
    item.setAttribute("data-retrieval-review-hit", String(index + 1));

    const left = document.createElement("div");
    const title = document.createElement("strong");
    title.textContent = hit.document?.title || "Evidence hit " + (index + 1);
    const snippet = document.createElement("span");
    snippet.textContent = textPreview(
      hit.chunk?.text_content || hit.evidence_items?.[0]?.statement,
      "No text preview returned for this hit."
    );
    left.append(title, snippet);

    const right = document.createElement("div");
    addMeta(right, "score", formatScore(hit.score));
    addMeta(right, "mode", retrievalMode(hit));
    addMeta(right, "chunk", shortId(hit.chunk?.id || hit.qdrant_payload?.chunk_id));
    addMeta(right, "document", shortId(hit.document?.id || hit.qdrant_payload?.document_id));
    addMeta(right, "source", hit.source?.name || shortId(hit.source?.id));
    addMeta(right, "evidence", String(hit.evidence_items?.length ?? 0));

    item.append(left, right);

    const evidenceItems = Array.isArray(hit.evidence_items) ? hit.evidence_items.slice(0, 2) : [];
    for (const evidenceItem of evidenceItems) {
      const evidenceLine = document.createElement("p");
      evidenceLine.className = "messageMeta";
      evidenceLine.textContent = "Evidence item: " + textPreview(evidenceItem.statement, evidenceItem.evidence_type || "recorded evidence");
      item.appendChild(evidenceLine);
    }

    return item;
  };

  const showSkeleton = () => {
    const wrap = document.createElement("div");
    wrap.className = "skeletonBlock";
    const line = document.createElement("span");
    line.className = "skeleton";
    const line2 = document.createElement("span");
    line2.className = "skeleton";
    const short = document.createElement("span");
    short.className = "skeleton skeletonShort";
    wrap.append(line, line2, short);
    results.appendChild(wrap);
    return wrap;
  };

  const showRetry = (label, handler) => {
    const box = document.createElement("div");
    box.className = "guidedManualActions";
    const btn = document.createElement("button");
    btn.type = "button";
    btn.textContent = label || "Retry";
    btn.addEventListener("click", handler);
    box.appendChild(btn);
    results.appendChild(box);
    return box;
  };

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    status.textContent = "Retrieving context";
    results.replaceChildren();
    results.removeAttribute("data-answer-status");
    results.removeAttribute("data-hit-count");
    lastPayload = null;
    lastQuestion = message.value.trim();
    lastRequest = { message: lastQuestion, limit: Number(limit.value || 5) };
    if (saveStatus) saveStatus.textContent = "Run retrieval before saving an answer record.";

    const skeleton = showSkeleton();
    try {
      const response = await fetch(apiBaseUrl + "/chat/retrieval-preview", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(lastRequest)
      });

      if (!response.ok) {
        status.textContent = "Error: " + response.status + " " + response.statusText;
        if (skeleton && skeleton.parentNode) skeleton.parentNode.removeChild(skeleton);
        const error = document.createElement("article");
        error.className = "item evidenceItem";
        error.setAttribute("data-retrieval-review-error", "");
        const body = document.createElement("div");
        const title = document.createElement("strong");
        title.textContent = "Retrieval failed";
        const detail = document.createElement("span");
        detail.textContent = "Unable to retrieve local evidence. Check that the local API and memory are ready, then retry.";
        body.append(title, detail);
        error.appendChild(body);
        results.appendChild(error);
        showRetry("Retry retrieval", () => form.requestSubmit());
        return;
      }

      const payload = await response.json();
      lastPayload = payload;
      const hits = payload.retrieval_context?.hits ?? [];
      const answerStatus = answerStatusFor(payload, hits);
      status.textContent = "answer_status: " + answerStatus + " | hits: " + hits.length;
      results.setAttribute("data-answer-status", answerStatus);
      results.setAttribute("data-hit-count", String(hits.length));
      if (skeleton && skeleton.parentNode) skeleton.parentNode.removeChild(skeleton);
      results.appendChild(renderReviewSummary(payload, hits));
      results.appendChild(renderAnswerPacket(payload, hits));

      if (hits.length > 0) {
        hits.forEach((hit, index) => results.appendChild(renderHit(hit, index)));
      } else {
        const empty = document.createElement("article");
        empty.className = "item evidenceItem";
        const body = document.createElement("div");
        const title = document.createElement("strong");
        title.textContent = "No matching evidence";
        const detail = document.createElement("span");
        detail.textContent = "Try a more specific question, or add/process data under Chat → Add Data.";
        body.append(title, detail);
        empty.appendChild(body);
        results.appendChild(empty);
      }
    } catch (error) {
      status.textContent = "Error: " + (error instanceof Error ? error.message : "Unknown error");
      if (skeleton && skeleton.parentNode) skeleton.parentNode.removeChild(skeleton);
      const item = document.createElement("article");
      item.className = "item evidenceItem";
      item.setAttribute("data-retrieval-review-error", "");
      const body = document.createElement("div");
      const title = document.createElement("strong");
      title.textContent = "Retrieval failed";
      const detail = document.createElement("span");
      detail.textContent = "No evidence-backed review is available until the local API responds. Retry when ready.";
      body.append(title, detail);
      item.appendChild(body);
      results.appendChild(item);
      showRetry("Retry retrieval", () => form.requestSubmit());
    }
  });

  saveButton?.addEventListener("click", async () => {
    if (!lastPayload || !lastQuestion) {
      if (saveStatus) saveStatus.textContent = "Ask over evidence before saving an answer record.";
      return;
    }
    saveButton.disabled = true;
    if (saveStatus) saveStatus.textContent = "Saving answer record";
    try {
      const response = await fetch(apiBaseUrl + "/evidence-answers", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(buildAnswerRecordPayload())
      });
      const payload = await response.json().catch(() => ({}));
      if (!response.ok) {
        throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
      }
      if (saveStatus) saveStatus.textContent = "Saved answer record " + (payload.id || "recorded") + ". Refresh Chat to see it in history.";
    } catch (error) {
      if (saveStatus) saveStatus.textContent = "Answer record save failed. Retry when ready.";
      const footerRetry = document.createElement("div");
      footerRetry.className = "guidedManualActions";
      const btn = document.createElement("button");
      btn.type = "button";
      btn.textContent = "Retry save";
      btn.addEventListener("click", async () => {
        footerRetry.remove();
        saveButton.click();
      });
      footerRetry.appendChild(btn);
      results.appendChild(footerRetry);
    }
    saveButton.disabled = false;
  });
})();
`;

  return (
    <section className="panel chatPreviewPanel chatEnginePanel">
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Ask over local evidence</p>
          <h2><HelpHeading term="chatRetrievalPreview">Ask Over Evidence</HelpHeading></h2>
        </div>
        <span className="statusText" data-chat-preview-status>answer_status: not_generated</span>
      </div>
      <form className="previewForm" data-chat-preview-form data-api-base-url={browserApiBaseUrl}>
        <label>
          <span>Question or request</span>
          <small>Ask about local evidence. Example for home use: "What does this document say about my bill?" Example for coders: "What failed in this build log? Cite the evidence."</small>
          <textarea data-chat-preview-message name="message" rows={3} placeholder="Ask a question or request an action..." defaultValue="What did I upload today?" />
        </label>
        <label>
          <span>Evidence limit</span>
          <small>How many matching local chunks to show. Example: 5.</small>
          <input data-chat-preview-limit name="limit" type="number" min="1" max="50" defaultValue="5" />
        </label>
        <button type="submit">Ask over evidence</button>
      </form>
      <div className="guidedManualActions">
        <button type="button" data-chat-save-answer>Save answer record</button>
        <span data-chat-save-answer-status>Run retrieval before saving. Saved records preserve history and do not change evidence.</span>
      </div>
      <div className="previewNote">
        Retrieval context only until saved. <TermHelp term="noExternalModel" label="No external model" /> answer, hidden reasoning, external model call, full chat memory, or action execution.
        <span data-retrieval-review-guidance> Evidence-backed only when hits are present; empty results mean insufficient evidence, not proof the information does not exist.</span>
      </div>
      <div className="stack previewResults" data-chat-preview-results />
      <ClientScript script={script} />
    </section>
  );
}

