import type { AgentCapabilitiesResponse, HealthResponse, RuntimeLogsResponse, VectorCollectionStatus } from "../../types";
import {
  dedupeHealthChecksByLabel,
  plainHealthStatus,
  plainServiceName,
  searchIndexStatus,
  summarizeRuntime,
  visibleHealthChecks,
} from "../helpers";
import { TroubleshootingLogsPanel } from "../TroubleshootingLogsPanel";
import { SystemServiceCard } from "../SystemServiceCard";
import { SectionHeader } from "../../shell/SectionHeader";
import { ActionButton } from "../../ui/ActionButton";
import { DetailDrawer } from "../../ui/DetailDrawer";

export function SystemSection({
  health,
  vectorCollection,
  llmEnabled,
  llmModel,
  runtimeLogs,
  agentCapabilities,
}: {
  health: HealthResponse;
  vectorCollection: VectorCollectionStatus;
  llmEnabled: boolean;
  llmModel: string;
  runtimeLogs: RuntimeLogsResponse;
  agentCapabilities: AgentCapabilitiesResponse;
}) {
  const serviceChecks = dedupeHealthChecksByLabel(visibleHealthChecks(health.checks ?? {}));
  const searchIndex = searchIndexStatus(vectorCollection);
  const searchReachable = Boolean(vectorCollection.detail?.tcp_reachable);
  const runtimeSummary = summarizeRuntime(health);

  return (
    <section className="productSection" data-product-section="system" data-tab-panel="advanced" hidden id="advanced-diagnostics">
      <SectionHeader
        eyebrow="System"
        title="Health and services"
        lead={runtimeSummary}
        statusState={health.status}
      />
      <div className="productServiceGrid" id="search-status">
        <SystemServiceCard name="Web UI" label="Running" state="ok" />
        {serviceChecks.map(([name, check]) => {
          const plain = plainHealthStatus(check.status);
          return (
            <SystemServiceCard
              key={name}
              name={plainServiceName(name)}
              label={plain.label}
              state={plain.state}
              detail={check.detail}
            />
          );
        })}
        <SystemServiceCard
          name="Search"
          label={searchIndex.label}
          state={searchIndex.state}
          searchStatusLabel
          action={
            searchIndex.state !== "ok" ? (
              <ActionButton variant="ghost" data-product-vector-ensure>
                {searchReachable ? "Repair search" : "Check services first"}
              </ActionButton>
            ) : undefined
          }
        />
        <SystemServiceCard
          name="Local model"
          label={llmEnabled ? `Running (${llmModel})` : "Not configured"}
          state={llmEnabled ? "ok" : "unknown"}
        />
      </div>
      <TroubleshootingLogsPanel logs={runtimeLogs} />
      <DetailDrawer summary="Advanced diagnostics">
        <pre className="uiBreakLong">
          {JSON.stringify({ health, vector: vectorCollection, capabilities: agentCapabilities.runtime }, null, 2)}
        </pre>
      </DetailDrawer>
    </section>
  );
}