import type {
  ApiResult,
  ApprovalRecord,
  CollectionRunRecord,
  VectorCollectionStatus,
  WorkItemRecord,
} from "../../types";
import { formatDate, excerpt } from "../../helpers";
import { workActivityLabel, workStatusPlain } from "../helpers";
import { plainApprovalType } from "../plain-language";
import { PipelineOperationsPanel } from "../../PipelineOperationsPanel";
import { SectionHeader } from "../../shell/SectionHeader";
import { EmptyState } from "../../ui/EmptyState";
import { StatusBadge } from "../../ui/StatusBadge";
import { ActionButton } from "../../ui/ActionButton";
import { DetailDrawer } from "../../ui/DetailDrawer";

export function ActivitySection({
  workItems,
  collectionRuns,
  vectorCollection,
  recentWork,
  pendingApprovals,
  recentRuns,
}: {
  workItems: ApiResult<WorkItemRecord[]>;
  collectionRuns: ApiResult<CollectionRunRecord[]>;
  vectorCollection: ApiResult<VectorCollectionStatus>;
  recentWork: WorkItemRecord[];
  pendingApprovals: ApprovalRecord[];
  recentRuns: CollectionRunRecord[];
}) {
  return (
    <section className="productSection" data-product-section="activity" data-tab-panel="work" hidden id="work-processing">
      <SectionHeader eyebrow="Activity" title="What IGY6 is doing" />
      <PipelineOperationsPanel
        workItems={workItems}
        collectionRuns={collectionRuns}
        vectorCollection={vectorCollection}
      />
      <div className="productCardGrid" id="activity-work">
        {recentWork.length > 0 ? (
          recentWork.map((item) => {
            const status = workStatusPlain(item);
            return (
              <article className="productActivityCard uiOverflowSafe" key={item.id}>
                <div className="productEvidenceHead">
                  <h3 className="uiClampTitle">{workActivityLabel(item)}</h3>
                  <StatusBadge state={status.state} label={status.label} />
                </div>
                <p className="uiClampSummary">{status.next}</p>
                <span className="productMuted">Updated {formatDate(item.updated_at ?? item.created_at)}</span>
                {item.status === "failed" ? (
                  <ActionButton variant="primary" data-retry-failed-work data-work-item-id={item.id}>
                    Retry
                  </ActionButton>
                ) : null}
                <DetailDrawer summary="Technical details">
                  <pre className="uiBreakLong">
                    {JSON.stringify(
                      { id: item.id, work_type: item.work_type, status: item.status, error_message: item.error_message },
                      null,
                      2,
                    )}
                  </pre>
                </DetailDrawer>
              </article>
            );
          })
        ) : (
          <EmptyState label="No work recorded yet." />
        )}
      </div>
      <section className="productSubsection" id="approvals-panel">
        <h3>Approvals</h3>
        {pendingApprovals.length > 0 ? (
          <div className="productCardGrid">
            {pendingApprovals.map((approval) => (
              <article className="productCard uiOverflowSafe" key={approval.id}>
                <strong className="uiClampTitle">{plainApprovalType(approval.request_type)}</strong>
                <span>Needs your decision</span>
                <ActionButton variant="ghost" data-product-go="settings" data-product-scroll="safety-audit">
                  Review in Settings
                </ActionButton>
              </article>
            ))}
          </div>
        ) : (
          <p className="productHint">No approvals waiting.</p>
        )}
      </section>
      {recentRuns.length > 0 ? (
        <section className="productSubsection">
          <h3>Recent collections</h3>
          <div className="productCardGrid">
            {recentRuns.map((run) => {
              const summary = run.summary_json ?? {};
              const aiSummary = typeof summary.ai_review_summary === "string" ? summary.ai_review_summary : null;
              return (
                <article className="productCard uiOverflowSafe" key={run.id}>
                  <strong>{run.status}</strong>
                  <span>{formatDate(run.created_at)}</span>
                  {aiSummary ? <p className="uiClampSummary">{excerpt(aiSummary, 160)}</p> : null}
                </article>
              );
            })}
          </div>
        </section>
      ) : null}
    </section>
  );
}