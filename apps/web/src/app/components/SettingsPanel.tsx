import type { EnvSettingsResponse, ApiResult } from "./types";
import { TERM_HELP } from "./constants";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";
import { StatusPill } from "./ui/StatusPill";
import { TermHelp } from "./ui/TermHelp";
import { HelpHeading } from "./ui/HelpHeading";
import { LocalLlmStatusPanel } from "./LocalLlmStatusPanel";

export function SettingsPanel({ envSettings }: { envSettings: ApiResult<EnvSettingsResponse> }) {
  const data = envSettings.data;
  const groupedSettings = data.groups.map((group) => ({
    ...group,
    settings: data.settings.filter((setting) => setting.group === group.key)
  }));
  const groupHelpTerms: Record<string, keyof typeof TERM_HELP> = {
    qdrant: "qdrant",
    neo4j: "neo4j",
    storage: "artifactStore",
    llm: "localLlm",
    policy: "externalModelPolicy"
  };
  const settingHelpTerms: Record<string, keyof typeof TERM_HELP> = {
    ENV_FILE_PATH: "ENV_FILE_PATH",
    ENV_BACKUP_DIR: "ENV_BACKUP_DIR",
    IGY6_DATA_ROOT: "IGY6_DATA_ROOT",
    QDRANT_CHUNK_VECTOR_SIZE: "QDRANT_CHUNK_VECTOR_SIZE",
    EXTERNAL_MODEL_POLICY_DEFAULT: "EXTERNAL_MODEL_POLICY_DEFAULT",
    APPROVAL_REQUIRED_DEFAULT: "APPROVAL_REQUIRED_DEFAULT",
    LLM_PROVIDER: "localLlm",
    OLLAMA_BASE_URL: "localLlm",
    OLLAMA_MODEL: "localLlm",
    LLM_TIMEOUT_SECONDS: "localLlm",
    LLM_EVIDENCE_REQUIRED: "localLlm",
    ARTIFACT_STORE_PATH: "artifactStore",
    EXPORT_STORE_PATH: "exportStore"
  };
  const script = `
(() => {
  const root = document.querySelector("[data-settings-env]");
  if (!root) return;
  if (root.getAttribute("data-settings-wired") === "true") return;
  root.setAttribute("data-settings-wired", "true");

  const verifyButton = root.querySelector("[data-settings-verify]");
  const saveButton = root.querySelector("[data-settings-save]");
  const resultPanel = root.querySelector("[data-settings-result]");
  const changedPanel = root.querySelector("[data-settings-changed]");
  const warningPanel = root.querySelector("[data-settings-warnings]");
  const backupPanel = root.querySelector("[data-settings-backup]");
  const tokenInput = root.querySelector("[data-settings-token]");
  let verifiedToken = "";
  let verifiedPayload = "";

  const showJson = (node, label, payload) => {
    if (!node) return;
    node.textContent = label + "\\n" + JSON.stringify(payload, null, 2);
  };

  const collectChanges = () => {
    const values = {};
    root.querySelectorAll("[data-env-key]").forEach((field) => {
      const key = field.getAttribute("data-env-key");
      const secret = field.getAttribute("data-secret") === "true";
      const readOnly = field.getAttribute("data-read-only") === "true";
      if (!key || readOnly) return;

      if (secret) {
        const replace = root.querySelector("[data-secret-replace='" + key + "']");
        if (replace?.checked && field.value !== "") {
          values[key] = field.value;
        }
        return;
      }

      const current = field.getAttribute("data-current") ?? "";
      if (field.value !== current) {
        values[key] = field.value;
      }
    });
    return values;
  };

  const clearVerified = () => {
    verifiedToken = "";
    verifiedPayload = "";
    if (tokenInput) tokenInput.value = "";
    if (saveButton) saveButton.disabled = true;
  };

  root.querySelectorAll("[data-env-key], [data-secret-replace]").forEach((field) => {
    field.addEventListener("input", clearVerified);
    field.addEventListener("change", clearVerified);
  });

  verifyButton?.addEventListener("click", async () => {
    clearVerified();
    const values = collectChanges();
    showJson(resultPanel, "Verifying dry run", { changed_keys: Object.keys(values) });
    try {
      const response = await fetch("/api/settings/env/verify", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ values, actor_id: "local-owner" })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Dry-run result" : "Dry-run request failed", payload);
      showJson(changedPanel, "Changed keys", payload.changed_keys ?? Object.keys(values));
      showJson(warningPanel, "Warnings", payload.warnings ?? []);
      if (response.ok && payload.passed && payload.verification_token) {
        verifiedToken = payload.verification_token;
        verifiedPayload = JSON.stringify(values);
        if (tokenInput) tokenInput.value = verifiedToken;
        if (saveButton) saveButton.disabled = false;
      }
    } catch (error) {
      showJson(resultPanel, "Dry-run error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });

  saveButton?.addEventListener("click", async () => {
    const values = collectChanges();
    if (!verifiedToken || JSON.stringify(values) !== verifiedPayload) {
      clearVerified();
      showJson(resultPanel, "Save blocked", { detail: "Current edits do not match the latest passing dry run." });
      return;
    }
    showJson(resultPanel, "Saving verified candidate", { changed_keys: Object.keys(values) });
    try {
      const response = await fetch("/api/settings/env/apply", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ values, verification_token: verifiedToken, actor_id: "local-owner" })
      });
      const payload = await response.json();
      showJson(resultPanel, response.ok ? "Save result" : "Save failed", payload);
      showJson(backupPanel, "Backup", { backup_path: payload.backup_path, restart_required: payload.restart_required, restart_notes: payload.restart_notes });
      if (response.ok && payload.saved) {
        if (saveButton) saveButton.disabled = true;
      }
    } catch (error) {
      showJson(resultPanel, "Save error", { detail: error instanceof Error ? error.message : "Unknown error" });
    }
  });
})();
`;

  return (
    <section className="panel settingsPanel tabContent" id="settings" data-settings-env data-tab-panel="settings">
      <div className="panelHeader">
        <div>
          <p className="eyebrow">Local-only configuration</p>
          <h2>Settings</h2>
        </div>
        <div className="topStatus">
          <StatusPill state={data.file_status.exists ? "env-mounted" : "env-missing"} />
          <StatusPill state={data.file_status.writable ? "writable" : "read-only"} />
        </div>
      </div>

      <div className="settingsNotice">
        <strong>Verify before save.</strong>
        <span>Edits are dry-run validated before `.env` is written. Saved settings may require Docker stack restart/recreate before taking effect.</span>
      </div>
      {envSettings.error ? <p className="errorText">{envSettings.error}</p> : null}

      <LocalLlmStatusPanel envSettings={envSettings} context="settings" />

      <section className="settingsMeta">
        <article><span>Env file</span><strong>{data.file_status.path}</strong></article>
        <article><span>Backup dir</span><strong>{data.file_status.backup_dir}</strong></article>
        <article><span>Format</span><strong>{data.file_status.output_format}</strong></article>
      </section>

      {data.warnings.length > 0 ? (
        <div className="settingsWarnings">
          {data.warnings.map((warning) => <span key={warning}>{warning}</span>)}
        </div>
      ) : null}

      <div className="settingsGroups">
        {groupedSettings.map((group) => (
          <section className="settingsGroup" key={group.key}>
            <h3>{groupHelpTerms[group.key] ? <HelpHeading term={groupHelpTerms[group.key]}>{group.label}</HelpHeading> : group.label}</h3>
            <div className="settingsRows">
              {group.settings.map((setting) => (
                <article className="settingRow" key={setting.key}>
                  <div className="settingInfo">
                    <strong>
                      {settingHelpTerms[setting.key] ? (
                        <TermHelp term={settingHelpTerms[setting.key]} label={setting.key} />
                      ) : setting.key}
                    </strong>
                    <span>{setting.description}</span>
                    <div className="messageMeta">
                      {setting.secret ? <StatusPill state="secret-masked" /> : null}
                      {setting.read_only ? <StatusPill state="read-only" /> : null}
                      {setting.restart_required ? <StatusPill state="restart-likely" /> : null}
                    </div>
                  </div>
                  <div className="settingControl">
                    {setting.secret ? (
                      <>
                        <label className="checkLine">
                          <input type="checkbox" data-secret-replace={setting.key} disabled={setting.read_only} />
                          Replace value
                        </label>
                        <input
                          type="password"
                          placeholder={setting.has_value ? setting.masked_value ?? "masked" : "empty"}
                          data-env-key={setting.key}
                          data-secret="true"
                          data-read-only={setting.read_only ? "true" : "false"}
                          disabled={setting.read_only}
                        />
                      </>
                    ) : (
                      <input
                        defaultValue={setting.value ?? ""}
                        data-current={setting.value ?? ""}
                        data-env-key={setting.key}
                        data-secret="false"
                        data-read-only={setting.read_only ? "true" : "false"}
                        readOnly={setting.read_only}
                      />
                    )}
                  </div>
                </article>
              ))}
            </div>
          </section>
        ))}
      </div>

      {data.unmanaged.length > 0 ? (
        <section className="settingsGroup unmanagedSettings">
          <h3><HelpHeading term="permissionScope">Unmanaged read-only keys</HelpHeading></h3>
          <div className="settingsRows">
            {data.unmanaged.map((item) => (
              <article className="settingRow" key={item.key}>
                <div className="settingInfo">
                  <strong>{item.key}</strong>
                  <span>Unknown key preserved by backend, not editable from this UI.</span>
                </div>
                <div className="settingControl">
                  <input readOnly value={item.masked_value} />
                </div>
              </article>
            ))}
          </div>
        </section>
      ) : null}

      <section className="settingsActions">
        <button type="button" data-settings-verify>Verify Dry Run</button>
        <button type="button" data-settings-save disabled>Save Settings</button>
        <input data-settings-token readOnly placeholder="verification token appears after passing dry run" />
      </section>

      <section className="settingsResultGrid">
        <pre data-settings-result>Dry-run result appears here.</pre>
        <pre data-settings-changed>Changed keys appear here.</pre>
        <pre data-settings-warnings>Warnings appear here.</pre>
        <pre data-settings-backup>Backup path appears after save.</pre>
      </section>
      <ClientScript script={script} />
    </section>
  );
}

