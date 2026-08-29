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
