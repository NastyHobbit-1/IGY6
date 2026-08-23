import type { SourceRecord, EvidenceItemRecord, EvidenceAnswerRecord, PatternRecord, FeedbackRecord, OutcomeRecord, ApiResult } from "./types";
import { formatDate, excerpt, uniqueStringValues } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

type PatternCandidate = {
  type: string;
  summary: string;
  evidenceIds: string[];
  supportCount: number;
  confidence: number | null;
  status: string;
  unverified: string;
  nextAction: string;
  source: string;
};

export function BaselinePatternExpansionPanel({
  patterns,
  evidenceItems,
  sources,
  evidenceAnswers,
  outcomes,
  feedback
}: {
  patterns: ApiResult<PatternRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  sources: ApiResult<SourceRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  outcomes: ApiResult<OutcomeRecord[]>;
  feedback: ApiResult<FeedbackRecord[]>;
}) {
  const browserApiBaseUrl = "/api";
  const sourceById = new Map(sources.data.map((source) => [source.id, source]));
  const evidenceById = new Map(evidenceItems.data.map((item) => [item.id, item]));
  const sourceCountFor = (ids: string[]) => new Set(ids.map((id) => evidenceById.get(id)?.source_id).filter(Boolean)).size;
  const confidenceFor = (supportCount: number, max = 85) => Math.min(max, 45 + supportCount * 10);
  const byType = new Map<string, EvidenceItemRecord[]>();
  evidenceItems.data.forEach((item) => {
    const key = item.evidence_type || "unknown";
    byType.set(key, [...(byType.get(key) ?? []), item]);
  });
  const byStatement = new Map<string, EvidenceItemRecord[]>();
  evidenceItems.data.forEach((item) => {
    const key = item.statement.toLowerCase().replace(/\s+/g, " ").trim().slice(0, 180);
    if (!key) return;
    byStatement.set(key, [...(byStatement.get(key) ?? []), item]);
  });
  const negativeLabels = new Set(["wrong", "not_useful", "partial", "inconclusive", "incomplete", "rejected", "weak"]);
  const positiveLabels = new Set(["correct", "useful", "verified", "trusted"]);
  const negativeSignals = [
    ...feedback.data.filter((event) => negativeLabels.has(event.label)).map((event) => ({ id: event.id, label: event.label, target: `${event.target_type}:${event.target_id}` })),
    ...outcomes.data.filter((outcome) => negativeLabels.has(outcome.outcome_status)).map((outcome) => ({ id: outcome.id, label: outcome.outcome_status, target: `${outcome.target_type}:${outcome.target_id}` }))
  ];
  const positiveSignals = [
    ...feedback.data.filter((event) => positiveLabels.has(event.label)).map((event) => ({ id: event.id, label: event.label, target: `${event.target_type}:${event.target_id}` })),
    ...outcomes.data.filter((outcome) => positiveLabels.has(outcome.outcome_status)).map((outcome) => ({ id: outcome.id, label: outcome.outcome_status, target: `${outcome.target_type}:${outcome.target_id}` }))
  ];
  const repeatedSignalCandidate = (signals: Array<{ id: string; label: string; target: string }>, type: string, label: string): PatternCandidate | null => {
    const counts = new Map<string, number>();
    signals.forEach((signal) => counts.set(signal.label, (counts.get(signal.label) ?? 0) + 1));
    const repeated = Array.from(counts.entries()).filter(([, count]) => count > 1).sort((a, b) => b[1] - a[1])[0];
    if (!repeated) return null;
    return {
      type,
      summary: `${repeated[1]} review signals repeat label ${repeated[0]}.`,
      evidenceIds: [],
      supportCount: repeated[1],
      confidence: confidenceFor(repeated[1], 75),
      status: "review-only",
      unverified: "This is grouped feedback/outcome metadata, not proof of a causal method pattern.",
      nextAction: label,
      source: "loaded feedback/outcome records"
    };
  };
  const candidates: PatternCandidate[] = [];
  Array.from(byType.entries()).forEach(([type, items]) => {
    if (items.length < 2) return;
    candidates.push({
      type: "recurrence",
      summary: `${items.length} evidence items share evidence type ${type}.`,
      evidenceIds: items.slice(0, 10).map((item) => item.id),
      supportCount: items.length,
      confidence: confidenceFor(items.length),
      status: "candidate",
      unverified: "The repeated type is a count signal only; it has not been statistically validated.",
      nextAction: "Review the cited evidence and decide whether the recurrence is meaningful.",
      source: "local evidence items"
    });
  });
  const missingInfo = evidenceAnswers.data.flatMap((answer) => answer.missing_information ?? []);
  if (missingInfo.length > 0) {
    const answerEvidence = evidenceAnswers.data.flatMap((answer) => answer.evidence_item_ids ?? []);
    candidates.push({
      type: "missing_information_gap",
      summary: `${missingInfo.length} saved answer missing-information note(s) indicate evidence gaps.`,
      evidenceIds: uniqueStringValues(answerEvidence, 10),
      supportCount: missingInfo.length,
      confidence: confidenceFor(missingInfo.length, 70),
      status: answerEvidence.length > 0 ? "candidate" : "review-only",
      unverified: "Missing local evidence does not prove real-world absence.",
      nextAction: "Add focused manual text, conversation history, or user observations that address the missing notes.",
      source: "saved evidence answer records"
    });
  }
  Array.from(byStatement.entries()).forEach(([, items]) => {
    const evidenceIds = items.map((item) => item.id);
    const sourceCount = sourceCountFor(evidenceIds);
    if (sourceCount < 2) return;
    candidates.push({
      type: "cross_source_agreement",
      summary: `${items.length} matching or near-matching statements appear across ${sourceCount} sources.`,
      evidenceIds: evidenceIds.slice(0, 10),
      supportCount: items.length,
      confidence: 60,
      status: "candidate",
      unverified: "Matching text may be agreement, duplication, or copied material.",
      nextAction: "Inspect source trust and decide whether this is agreement or duplicated evidence.",
      source: "normalized evidence statements"
    });
    candidates.push({
      type: "cross_source_conflict",
      summary: `${items.length} related statements appear across ${sourceCount} sources and may need conflict review.`,
      evidenceIds: evidenceIds.slice(0, 10),
      supportCount: items.length,
      confidence: 55,
      status: "candidate",
      unverified: "The UI has not proven contradiction; it is a prompt to compare sources.",
      nextAction: "Open evidence details and compare source context before treating this as a conflict.",
      source: "normalized evidence statements"
    });
  });
  const configGroups = new Map<string, EvidenceItemRecord[]>();
  evidenceItems.data.forEach((item) => {
    const normalized = item.statement.toLowerCase().replace(/\s+/g, " ").trim();
    if (!/(config|configuration|setting|version|feature flag|threshold)/.test(normalized)) return;
    const key = normalized.split(/[:=\-]/)[0]?.trim().slice(0, 80);
    if (!key || key.length < 3) return;
    configGroups.set(key, [...(configGroups.get(key) ?? []), item]);
  });
  Array.from(configGroups.entries()).forEach(([key, items]) => {
    const distinctStatements = new Set(items.map((item) => item.statement.toLowerCase().replace(/\s+/g, " ").trim()));
    if (distinctStatements.size < 2) return;
    candidates.push({
      type: "configuration_drift",
      summary: `Configuration-like evidence for ${key} differs across ${items.length} records.`,
      evidenceIds: items.slice(0, 10).map((item) => item.id),
      supportCount: items.length,
      confidence: 55,
      status: "candidate",
      unverified: "This is keyword grouping, not a full configuration parser.",
      nextAction: "Inspect source context and verify whether the setting actually drifted.",
      source: "configuration-like evidence statements"
    });
  });
  const anomalyItems = evidenceItems.data.filter((item) => /(anomaly|unexpected|outlier|spike|regression|unusual|sudden|abnormal)/i.test(item.statement));
  if (anomalyItems.length > 0) {
    candidates.push({
      type: "anomaly_signal",
      summary: `${anomalyItems.length} evidence item(s) contain anomaly or unexpected-state language.`,
      evidenceIds: anomalyItems.slice(0, 10).map((item) => item.id),
      supportCount: anomalyItems.length,
      confidence: 50,
      status: "candidate",
      unverified: "This is keyword matching, not statistical anomaly detection.",
      nextAction: "Review the evidence and supporting source before treating this as an anomaly.",
      source: "local evidence statements"
    });
  }
  const failedAdvice = repeatedSignalCandidate(negativeSignals, "failed_advice_recurrence", "Open Outcome Learning Summary and propose an improvement candidate if this repeats.");
  const successfulMethod = repeatedSignalCandidate(positiveSignals, "successful_method_recurrence", "Keep recording outcomes; do not auto-promote the method without review.");
  if (failedAdvice) candidates.push(failedAdvice);
  if (successfulMethod) candidates.push(successfulMethod);
  const supportedCategories = ["recurrence", "missing_information_gap", "cross_source_agreement", "cross_source_conflict", "configuration_drift", "anomaly_signal", "failed_advice_recurrence", "successful_method_recurrence"];
  const candidateOptions = candidates.filter((candidate) => candidate.evidenceIds.length > 0);
  const candidateOptionsJson = JSON.stringify(candidateOptions).replace(/</g, "\\u003c");
  const savedPatternDetails = patterns.data.map((pattern) => ({
    pattern,
    sourceNames: uniqueStringValues((pattern.evidence_ids ?? []).map((id) => {
      const sourceId = evidenceById.get(id)?.source_id;
      return sourceId ? sourceById.get(sourceId)?.name ?? sourceId : "";
    }), 4),
    unverified: pattern.metadata_json?.unverified_note as string | undefined
  }));
  const script = `
(() => {
  const root = document.querySelector("[data-baseline-pattern-expansion]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const form = root.querySelector("[data-pattern-create-form]");
  const detect = root.querySelector("[data-pattern-detect-expanded]");
  const result = root.querySelector("[data-pattern-expansion-result]");
  const candidates = JSON.parse(root.querySelector("[data-pattern-candidates-json]")?.textContent || "[]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
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
      [["pattern", payload.id], ["type", payload.pattern_type], ["status", payload.status]].forEach(([label, detail]) => {
        const term = document.createElement("dt");
        term.textContent = label;
        const description = document.createElement("dd");
        description.textContent = detail || "not returned";
        details.append(term, description);
      });
      result.appendChild(details);
    }
  };
  const postJson = async (path, body) => {
    const response = await fetch(apiBaseUrl + path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body)
    });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(response.status + " " + response.statusText + ": " + JSON.stringify(payload));
    return payload;
  };
  form?.addEventListener("submit", async (event) => {
    event.preventDefault();
    const candidate = candidates.find((item) => item.type === value("pattern_candidate_type"));
    if (!candidate || !Array.isArray(candidate.evidenceIds) || candidate.evidenceIds.length === 0) {
      show("Pattern not persisted", "This candidate has no evidence IDs, so it remains review-only.");
      return;
    }
    try {
      const payload = await postJson("/analysis/patterns", {
        pattern_type: candidate.type,
        summary: candidate.summary,
        evidence_ids: candidate.evidenceIds,
        confidence: candidate.confidence,
        status: "candidate",
        actor_id: "local-owner",
        metadata_json: {
          created_from: "baseline_pattern_expansion_mvp",
          support_count: candidate.supportCount,
          unverified_note: candidate.unverified,
          safe_next_action: candidate.nextAction,
          advanced_statistical_validation: false,
          forecasting: false,
          future_behavior_modified: false
        }
      });
      show("Pattern candidate saved", "Review the saved pattern before relying on it. No forecasting or behavior change occurred.", payload);
    } catch (error) {
      show("Pattern save failed", String(error));
    }
  });
  detect?.addEventListener("click", async () => {
    try {
      const payload = await postJson("/analysis/patterns/detect-baseline", {
        recurrence_threshold: Number(value("recurrence_threshold") || 3),
        actor_id: "local-owner"
      });
      show("Baseline detector finished", "Baseline detector ran for recurrence, missing-information gaps, agreement/conflict, configuration drift, anomaly signals, and outcome recurrence. Reload to inspect saved candidates.", { id: Array.isArray(payload.patterns) ? payload.patterns.length + " candidates" : "detector", pattern_type: "baseline", status: "recorded" });
    } catch (error) {
      show("Baseline detector failed", String(error));
    }
  });
})();
`;

  return (
    <section
      className="panel baselinePatternExpansion"
      data-baseline-pattern-expansion
      data-api-base-url={browserApiBaseUrl}
    >
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Pattern review</p>
          <h2>Baseline Pattern Expansion</h2>
        </div>
        <StatusPill state="review-not-statistics" />
      </div>
      <div className="guidedManualNotice">
        <strong>Baseline signals only.</strong>
        <span>Patterns are review prompts from existing local records. They do not provide advanced statistical validation, forecasting, statistical anomaly detection, or automatic behavior changes.</span>
      </div>
      <section className="metrics compact" aria-label="Supported pattern categories">
        {supportedCategories.map((category) => (
          <article key={category}><span>{category}</span><strong>{patterns.data.filter((pattern) => pattern.pattern_type === category).length + candidates.filter((candidate) => candidate.type === category).length}</strong></article>
        ))}
      </section>
      <section className="split">
        <div>
          <div className="subHeader"><h3>Saved Patterns</h3>{patterns.error ? <span className="errorText">{patterns.error}</span> : null}</div>
          <div className="stack">
            {savedPatternDetails.slice(0, 8).map(({ pattern, sourceNames, unverified }) => (
              <article className="item evidenceItem" key={pattern.id}>
                <div>
                  <strong>{pattern.pattern_type}</strong>
                  <span>{excerpt(pattern.summary, 150)}</span>
                  <span>Evidence: {(pattern.evidence_ids ?? []).length} · sources: {sourceNames.length > 0 ? sourceNames.join(", ") : "not resolved"}</span>
                  <span>Unverified: {unverified ?? "Review evidence before relying on this pattern."}</span>
                </div>
                <div>
                  <StatusPill state={pattern.status} />
                  <span>{pattern.confidence === null ? "support not scored" : `confidence ${pattern.confidence}%`}</span>
                  <span>{formatDate(pattern.created_at)}</span>
                </div>
              </article>
            ))}
          </div>
          {savedPatternDetails.length === 0 ? <EmptyState label="No saved baseline patterns yet." /> : null}
        </div>
        <div>
          <div className="subHeader"><h3>Detected Review Candidates</h3></div>
          <div className="stack">
            {candidates.slice(0, 10).map((candidate, index) => (
              <article className="item evidenceItem" key={`${candidate.type}:${index}`}>
                <div>
                  <strong>{candidate.type}</strong>
                  <span>{candidate.summary}</span>
                  <span>Linked evidence: {candidate.evidenceIds.length} · source: {candidate.source}</span>
                  <span>Unverified: {candidate.unverified}</span>
                  <span>Next: {candidate.nextAction}</span>
                </div>
                <div>
                  <StatusPill state={candidate.status} />
                  <span>{candidate.confidence === null ? "unscored" : `confidence ${candidate.confidence}%`}</span>
                </div>
              </article>
            ))}
          </div>
          {candidates.length === 0 ? <EmptyState label="No candidate pattern signals detected from loaded records." /> : null}
        </div>
      </section>
      <form className="guidedManualForm" data-pattern-create-form>
        <label>
          <span>Candidate to save</span>
          <select name="pattern_candidate_type" disabled={candidateOptions.length === 0}>
            {candidateOptions.map((candidate, index) => (
              <option key={`${candidate.type}:${index}`} value={candidate.type}>{candidate.type} · {candidate.supportCount} support</option>
            ))}
          </select>
        </label>
        <label>
          <span>Recurrence threshold</span>
          <input name="recurrence_threshold" type="number" min="2" max="20" defaultValue="3" />
        </label>
        <div className="guidedManualActions">
          <button type="submit" disabled={candidateOptions.length === 0}>Save candidate pattern</button>
          <button type="button" data-pattern-detect-expanded>Run existing baseline detector</button>
          <span>Saving requires linked evidence IDs. Review-only metadata patterns stay visible without persistence.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-pattern-expansion-result>
        <strong>{candidateOptions.length > 0 ? "Pattern candidates available" : "No persistable candidate selected"}</strong>
        <span>Unsupported states remain review-only; weak evidence is not hidden.</span>
      </div>
      <DomJsonScript marker="data-pattern-candidates-json" json={candidateOptionsJson} />
      <ClientScript script={script} />
    </section>
  );
}

