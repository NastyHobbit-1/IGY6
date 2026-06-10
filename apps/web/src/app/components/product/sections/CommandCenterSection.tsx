import type { AuditEventRecord } from "../../types";
import { formatDate } from "../../helpers";
import { plainAuditLabel } from "../plain-language";
import { ActionButton } from "../../ui/ActionButton";
import { ChatCommandPanel } from "../ChatCommandPanel";
import { ChatRetrievalPreview } from "../../ChatRetrievalPreview";
import { DetailDrawer } from "../../ui/DetailDrawer";

export function CommandCenterSection({
  sourcesCount,
  evidenceCount,
  activeWorkCount,
  searchLabel,
  pendingApprovals,
  chunkCount,
  workItemCount,
  vectorReady,
  llmEnabled,
  llmModel,
  recentAudit,
}: {
  sourcesCount: number;
  evidenceCount: number;
  activeWorkCount: number;
  searchLabel: string;
  pendingApprovals: number;
  chunkCount: number;
  workItemCount: number;
  vectorReady: boolean;
  llmEnabled: boolean;
  llmModel: string;
  recentAudit: AuditEventRecord[];
}) {
  return (
    <section
      className="productSection productSectionActive"
      data-product-section="command"
      data-tab-panel="results"
      id="assistant"
    >
      <header className="productHero">
        <div>
          <p className="productEyebrow">Command Center</p>
          <h1>Ask IGY6</h1>
          <p className="productLead">
            Ask questions over your local evidence, check status, open sections, or add information.
          </p>
        </div>
        <div className="productHeroStatus">
          <article><span>Sources</span><strong>{sourcesCount}</strong></article>
          <article><span>Evidence</span><strong>{evidenceCount}</strong></article>
          <article><span>Active work</span><strong>{activeWorkCount}</strong></article>
          <article><span>Search</span><strong data-search-status-label>{searchLabel}</strong></article>
        </div>
      </header>

      <div className="productQuickActions" aria-label="Quick actions">
        <ActionButton variant="chip" data-product-go="add" data-product-scroll="uploads-collection">
          Add information
        </ActionButton>
        <ActionButton variant="chip" data-product-go="evidence" data-product-scroll="evidence-panel">
          Browse evidence
        </ActionButton>
        <ActionButton variant="chip" data-product-go="gallery" data-product-scroll="image-gallery">
          Open gallery
        </ActionButton>
        <ActionButton variant="chip" data-product-go="activity" data-product-scroll="work-processing">
          See activity
        </ActionButton>
        <ActionButton variant="chip" data-product-go="system">
          System health
        </ActionButton>
        {pendingApprovals > 0 ? (
          <ActionButton variant="warn" data-product-go="activity" data-product-scroll="approvals-panel">
            {pendingApprovals} approval{pendingApprovals === 1 ? "" : "s"}
          </ActionButton>
        ) : null}
      </div>

      <ChatCommandPanel
        sourceCount={sourcesCount}
        evidenceCount={evidenceCount}
        chunkCount={chunkCount}
        workItemCount={workItemCount}
        pendingApprovals={pendingApprovals}
        vectorReady={vectorReady}
        llmEnabled={llmEnabled}
        llmModel={llmModel}
      />

      <DetailDrawer summary="What IGY6 found">
        <ChatRetrievalPreview />
      </DetailDrawer>

      {recentAudit.length > 0 ? (
        <section className="productCardGrid" aria-label="Recent activity">
          <h3>Recent activity</h3>
          {recentAudit.map((event) => (
            <article className="productCard uiOverflowSafe" key={event.id}>
              <strong className="uiClampTitle">{plainAuditLabel(event.event_type)}</strong>
              <span className="uiBreakLong">{event.decision ?? "recorded"} · {formatDate(event.created_at)}</span>
            </article>
          ))}
        </section>
      ) : null}
    </section>
  );
}