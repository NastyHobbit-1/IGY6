import type { EvidenceItemRecord, EvidenceAnswerRecord, ReportRecord, ApiResult } from "./types";
import { excerpt, shortRecordId } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { EmptyState } from "./ui/EmptyState";

export function BasicReportWorkflow({
  reports,
  evidenceItems,
  evidenceAnswers,
  evidenceCount,
  documentCount,
  chunkCount
}: {
  reports: ApiResult<ReportRecord[]>;
  evidenceItems: ApiResult<EvidenceItemRecord[]>;
  evidenceAnswers: ApiResult<EvidenceAnswerRecord[]>;
  evidenceCount: number;
  documentCount: number;
  chunkCount: number;
}) {
  const browserApiBaseUrl = "/api";
  const reportReady = evidenceCount > 0 || documentCount > 0 || chunkCount > 0;
  const templateOptions = [
    {
      key: "evidence_brief",
      label: "Evidence brief",
      reportType: "evidence_review",
      sections: ["summary", "evidence-backed sections", "uncertainty and missing information", "citation appendix"]
    },
    {
      key: "decision_note",
      label: "Decision note",
      reportType: "decision_note",
      sections: ["decision context", "evidence support", "assumptions and uncertainty", "citation appendix"]
    },
    {
      key: "handoff",
      label: "Handoff",
      reportType: "handoff",
      sections: ["current state", "known evidence", "open gaps", "next safe actions", "citation appendix"]
    },
    {
      key: "inventory_summary",
      label: "Inventory summary",
      reportType: "summary",
      sections: ["inventory counts", "local boundaries", "citation appendix"]
    }
  ];
  const citationEvidence = evidenceItems.data.slice(0, 8);
  const citationAnswerIds = evidenceAnswers.data.slice(0, 4).map((answer) => answer.id);
  const templateJson = JSON.stringify(templateOptions).replace(/</g, "\\u003c");
  const citationEvidenceJson = JSON.stringify(citationEvidence.map((item) => ({
    id: item.id,
    source_id: item.source_id,
    document_id: item.document_id,
    chunk_id: item.chunk_id,
    preview: excerpt(item.statement, 120)
  }))).replace(/</g, "\\u003c");
  const citationAnswerJson = JSON.stringify(citationAnswerIds).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-basic-report-workflow]");
  if (!root) return;
  const apiBaseUrl = root.getAttribute("data-api-base-url");
  const templates = JSON.parse(root.querySelector("[data-report-template-json]")?.textContent || "[]");
  const citationEvidence = JSON.parse(root.querySelector("[data-report-citation-evidence-json]")?.textContent || "[]");
  const citationAnswerIds = JSON.parse(root.querySelector("[data-report-citation-answer-json]")?.textContent || "[]");
  const form = root.querySelector("[data-basic-report-form]");
  const result = root.querySelector("[data-basic-report-result]");
  const submit = root.querySelector("[data-basic-report-submit]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const checked = (name) => Boolean(root.querySelector("[name='" + name + "']")?.checked);
  const selectedTemplate = () => templates.find((item) => item.key === value("basic_report_template")) || templates[0] || { key: "inventory_summary", reportType: "summary", sections: [] };
  const renderNotes = () => {
    const template = selectedTemplate();
    const userNotes = value("basic_report_notes");
    return [
      "Template: " + template.label,
      "",
      "Planned sections:",
      ...(template.sections || []).map((section) => "- " + section),
      "",
      "Citation/evidence appendix:",
      ...(citationEvidence.length > 0 ? citationEvidence.map((item) => "- " + item.id + ": " + item.preview) : ["- No evidence IDs were loaded when the report was requested."]),
      "",
      "Linked answer records:",
      ...(citationAnswerIds.length > 0 ? citationAnswerIds.map((id) => "- " + id) : ["- none loaded"]),
      userNotes ? "" : null,
      userNotes ? "Owner notes:" : null,
      userNotes || null
    ].filter(Boolean).join("\\n");
  };
  const show = (state, message, payload) => {
    if (result) {
      result.innerHTML = "";
      const title = document.createElement("strong");
      title.textContent = state;
      const body = document.createElement("span");
      body.textContent = message;
      result.append(title, body);
      if (payload) {
        const details = document.createElement("dl");
        details.setAttribute("data-basic-report-status", "");
        [
          ["report", payload.id],
          ["status", payload.status],
          ["type", payload.report_type],
          ["artifact", payload.artifact_path || "not rendered"]
        ].forEach(([label, detail]) => {
          const term = document.createElement("dt");
          term.textContent = label;
          const description = document.createElement("dd");
          description.textContent = detail || "not returned";
          details.append(term, description);
        });
        result.appendChild(details);
      }
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
    if (submit) {
      submit.disabled = true;
      submit.textContent = "Creating...";
    }
    try {
      const template = selectedTemplate();
      const report = await postJson("/reports", {
        title: value("basic_report_title") || "Evidence inventory report",
        report_type: template.reportType || "summary",
        status: "requested",
        metadata_json: {
          created_from: "results_basic_report_workflow",
          template_key: template.key,
          template_sections: template.sections || [],
          export_format: "markdown",
          unsupported_exports: ["pdf"],
          citation_evidence_ids: citationEvidence.map((item) => item.id),
          linked_answer_record_ids: citationAnswerIds,
          evidence_items_visible: Number(root.getAttribute("data-evidence-count") || 0),
          documents_visible: Number(root.getAttribute("data-document-count") || 0),
          chunks_visible: Number(root.getAttribute("data-chunk-count") || 0)
        }
      });
      let finalReport = report;
      if (checked("basic_report_render")) {
        finalReport = await postJson("/reports/" + report.id + "/render", {
          notes: renderNotes()
        });
      }
      show(
        finalReport.status === "ready" ? "Report ready" : "Report created",
        finalReport.status === "ready"
          ? "IGY6 rendered a local markdown report artifact with template notes and citation IDs."
          : "IGY6 created the report metadata record. Render it from Advanced or rerun this workflow with rendering enabled.",
        finalReport
      );
    } catch (error) {
      show("Report workflow failed", String(error));
    } finally {
      if (submit) {
        submit.disabled = false;
        submit.textContent = "Create report";
      }
    }
  });
})();
`;

  return (
    <section
      className="guidedManualText"
      data-basic-report-workflow
      data-api-base-url={browserApiBaseUrl}
      data-evidence-count={evidenceCount}
      data-document-count={documentCount}
      data-chunk-count={chunkCount}
    >
      <div className="guidedManualNotice">
        <strong>Basic report workflow</strong>
        <span>
          Current reports render local markdown artifacts through existing routes. Templates add section guidance, uncertainty notes, and citation IDs; they do not read raw artifact contents, call external models, or create PDF exports.
        </span>
      </div>
      <form className="guidedManualForm" data-basic-report-form>
        <label>
          <span>Report title</span>
          <input name="basic_report_title" defaultValue="Evidence inventory report" />
        </label>
        <label>
          <span>Template</span>
          <select name="basic_report_template" defaultValue="evidence_brief">
            {templateOptions.map((template) => (
              <option key={template.key} value={template.key}>{template.label} · {template.reportType}</option>
            ))}
          </select>
        </label>
        <label>
          <span>Render notes</span>
          <textarea name="basic_report_notes" rows={2} placeholder="Optional local note for the rendered markdown report." />
        </label>
        <label className="checkLine">
          <input name="basic_report_render" type="checkbox" defaultChecked /> Render markdown artifact now
        </label>
        <div className="guidedManualActions">
          <button type="submit" data-basic-report-submit disabled={!reportReady}>Create report</button>
          <span>{reportReady ? "Uses existing /reports and /reports/:id/render routes for markdown export." : "Add supported text and wait for evidence before creating a useful report."}</span>
        </div>
      </form>
      <section className="stack" aria-label="Report citation appendix preview">
        {citationEvidence.slice(0, 4).map((item) => (
          <article className="item evidenceItem" key={`report-citation-${item.id}`}>
            <div><strong>{shortRecordId(item.id)}</strong><span>{excerpt(item.statement, 140)}</span></div>
            <div><StatusPill state={item.evidence_type} /><span>{item.confidence === null ? "unscored" : `${item.confidence}%`}</span></div>
          </article>
        ))}
      </section>
      {citationEvidence.length === 0 ? <EmptyState label="No evidence IDs are available for a citation appendix yet." /> : null}
      <div className="guidedManualResult" data-basic-report-result>
        <strong>{reports.data.length > 0 ? "Reports are available" : "No reports yet"}</strong>
        <span>{reports.data.length > 0 ? "Create a new metadata report or review recent reports below." : "Create a report after evidence exists, or keep using Ask over evidence."}</span>
      </div>
      <DomJsonScript marker="data-report-template-json" json={templateJson} />
      <DomJsonScript marker="data-report-citation-evidence-json" json={citationEvidenceJson} />
      <DomJsonScript marker="data-report-citation-answer-json" json={citationAnswerJson} />
      <ClientScript script={script} />
    </section>
  );
}
