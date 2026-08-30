import type { SourceRecord, ApprovalRecord, ApiResult } from "./types";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function UserObservationIngestion({ sources, approvals }: { sources: ApiResult<SourceRecord[]>; approvals: ApiResult<ApprovalRecord[]> }) {
  const browserApiBaseUrl = "/api";
