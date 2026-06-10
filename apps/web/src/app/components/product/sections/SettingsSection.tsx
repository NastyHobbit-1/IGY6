import type { ApiResult, ApprovalRecord, EnvSettingsResponse, RawArtifactRecord } from "../../types";
import { SettingsPanel } from "../../SettingsPanel";
import { StackRestartPanel } from "../../StackRestartPanel";
import { UserSecurityPanel } from "../../UserSecurityPanel";
import { SectionHeader } from "../../shell/SectionHeader";
import { DetailDrawer } from "../../ui/DetailDrawer";

export function SettingsSection({
  envSettings,
  approvals,
  artifactsCount,
}: {
  envSettings: ApiResult<EnvSettingsResponse>;
  approvals: ApprovalRecord[];
  artifactsCount: number;
}) {
  return (
    <section className="productSection" data-product-section="settings" data-tab-panel="settings" hidden id="settings">
      <SectionHeader eyebrow="Settings" title="Configuration and privacy" />
      <div className="productSettingsStack" id="safety-audit">
        <StackRestartPanel />
        <SettingsPanel envSettings={envSettings} />
        <UserSecurityPanel />
      </div>
      <DetailDrawer summary="Advanced: raw environment and audit JSON">
        <pre className="uiBreakLong" id="user-security">
          {JSON.stringify(
            { settings: envSettings.data.file_status, approvals: approvals.slice(0, 5), artifacts: artifactsCount },
            null,
            2,
          )}
        </pre>
      </DetailDrawer>
    </section>
  );
}