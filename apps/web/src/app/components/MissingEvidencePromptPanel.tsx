import type { SourceRecord, ChunkRecord, EvidenceItemRecord, EvidenceAnswerRecord, AgentTaskPlanRecord, ApiResult } from "./types";
import { excerpt, stringArrayFromUnknown, numberFromUnknown } from "./helpers";
import { StatusPill } from "./ui/StatusPill";

export function MissingEvidencePromptPanel({
  evidenceItems,
  chunks,
  sources,
  evidenceAnswers,
  taskPlans
}: {
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  chunks: ApiResult<ChunkRecord[]>;
  sources: ApiResult<SourceRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  taskPlans: ApiResult<AgentTaskPlanRecord[]>;
}) {
  const latestAnswer = evidenceAnswers.data[0];
  const latestTaskEvidence = taskPlans.data
    .map((plan) => plan.metadata_json?.evidence_summary)
    .find((summary): summary is Record<string, unknown> => Boolean(summary && typeof summary === "object" && !Array.isArray(summary)));
  const latestMissingInfo = [
    ...(latestAnswer?.missing_information ?? []),
    ...stringArrayFromUnknown(latestTaskEvidence?.missing_information)
  ].filter(Boolean);
  const latestRetrievalCount = latestAnswer?.retrieval_count ?? numberFromUnknown(latestTaskEvidence?.retrieved_count) ?? 0;
  const noEvidence = evidenceItems.data.length === 0 || chunks.data.length === 0;
  const insufficientAnswer = latestAnswer?.answer_status === "insufficient_evidence" || latestRetrievalCount === 0;
  const weakEvidence = !noEvidence && !insufficientAnswer && (latestMissingInfo.length > 0 || latestRetrievalCount > 0 && latestRetrievalCount < 3);
  const evidenceStatus = noEvidence || insufficientAnswer
    ? "insufficient-evidence"
    : weakEvidence
      ? "weak-evidence"
      : "evidence-available";
  const reason = evidenceStatus === "insufficient-evidence"
    ? "No matching local evidence has been retrieved yet, or the current evidence base has no processed chunks/evidence items."
    : evidenceStatus === "weak-evidence"
      ? "Some evidence exists, but the latest answer or task evidence check still reports missing information or a low hit count."
      : "Evidence is available. Continue checking citations and missing-information notes before relying on an answer.";
  const hasLocalProjectSource = sources.data.some((source) => source.source_type === "local_project" && source.enabled);
  const suggestedSources = [
    {
      type: "manual text upload",
      state: "supported",
      action: "Add a focused UTF-8 note, log, document excerpt, or export through Add Data."
    },
    {
      type: "conversation_history",
      state: "supported",
      action: "Import relevant prior conversation text through the guided conversation-history path."
    },
    {
      type: "user_observation",
      state: "supported",
      action: "Record owner-provided observations, decisions, preferences, corrections, or notes."
    },
    {
      type: "local_project",
      state: hasLocalProjectSource ? "supported-existing-source" : "not-suggested",
      action: hasLocalProjectSource
        ? "Use the existing scoped local_project source only for already-authorized local project material."
        : "Not suggested in the normal path until a scoped local_project source exists."
    }
  ];
  const nextAction = evidenceStatus === "evidence-available"
    ? "Open the answer packet and inspect citations/source trails before saving or reviewing."
    : "Open Add Data and add a supported local source that directly addresses the missing question.";

  return (
    <section className="panel missingEvidencePanel" data-missing-evidence-prompts>
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Evidence gaps</p>
          <h2>Missing Evidence Prompts</h2>
        </div>
        <StatusPill state={evidenceStatus} />
      </div>
      <div className="guidedManualNotice">
        <strong>{evidenceStatus === "evidence-available" ? "Evidence is available" : evidenceStatus === "weak-evidence" ? "Evidence may be incomplete" : "Insufficient evidence"}</strong>
        <span>{reason} Missing evidence is a local coverage gap; it is not a claim that the real-world information does not exist.</span>
      </div>
      <section className="metrics compact" aria-label="Missing evidence status">
        <article><span>Evidence items</span><strong>{evidenceItems.data.length}</strong></article>
        <article><span>Chunks</span><strong>{chunks.data.length}</strong></article>
        <article><span>Latest retrieved hits</span><strong>{latestRetrievalCount}</strong></article>
        <article><span>Missing-info notes</span><strong>{latestMissingInfo.length}</strong></article>
      </section>
      <div className="stack">
        <article className="item evidenceItem">
          <div>
            <strong>Missing information</strong>
            <span>{latestMissingInfo.length > 0 ? latestMissingInfo.slice(0, 3).join(" | ") : "No saved missing-information note is available yet. Ask over evidence to create one."}</span>
          </div>
          <div>
            <StatusPill state={evidenceStatus} />
            <span>{nextAction}</span>
          </div>
        </article>
        {suggestedSources.map((source) => (
          <article className="item evidenceItem" key={source.type}>
            <div>
              <strong>{source.type}</strong>
              <span>{source.action}</span>
            </div>
            <div>
              <StatusPill state={source.state} />
            </div>
          </article>
        ))}
      </div>
      <div className="guidedManualActions">
        <label htmlFor="tab-add-data">Open Add Data</label>
        <label htmlFor="tab-results">Return to Results</label>
        <span>No data is collected automatically. Browser/account/connector collection is not part of this prompt.</span>
      </div>
    </section>
  );
}

