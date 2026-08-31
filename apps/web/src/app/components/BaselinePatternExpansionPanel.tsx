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
