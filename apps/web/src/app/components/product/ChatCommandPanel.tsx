import { UnifiedChatHub } from "../UnifiedChatHub";

export function ChatCommandPanel({
  sourceCount,
  evidenceCount,
  chunkCount,
  workItemCount,
  pendingApprovals,
  vectorReady,
  llmEnabled,
  llmModel,
}: {
  sourceCount: number;
  evidenceCount: number;
  chunkCount: number;
  workItemCount: number;
  pendingApprovals: number;
  vectorReady: boolean;
  llmEnabled: boolean;
  llmModel: string;
}) {
  return (
    <div className="productChatStage chatFirstShell">
      <UnifiedChatHub
        sourceCount={sourceCount}
        evidenceCount={evidenceCount}
        chunkCount={chunkCount}
        workItemCount={workItemCount}
        pendingApprovals={pendingApprovals}
        vectorReady={vectorReady}
        llmEnabled={llmEnabled}
        llmModel={llmModel}
      />
    </div>
  );
}