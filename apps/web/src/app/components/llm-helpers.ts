import type { EnvSettingsResponse } from "./types";

type LlmDisplay = {
  provider: string;
  baseUrl: string;
  model: string;
  timeout: string;
  evidenceRequired: string;
  enabledState: string;
  answerMode: string;
  routingState: string;
  fallbackState: string;
  externalUse: string;
  healthStatus: string;
  healthDetail: string;
  guidance: string;
  limitations: string[];
  rawDiagnostics: Record<string, string>;
};

export function buildLlmDisplay(data: EnvSettingsResponse): LlmDisplay {
  const provider = settingValue(data, "LLM_PROVIDER", "none") || "none";
  const baseUrl = redactLlmUrl(settingValue(data, "OLLAMA_BASE_URL", "http://host.docker.internal:11434"));
  const model = settingValue(data, "OLLAMA_MODEL", "") || "not selected";
  const timeout = settingValue(data, "LLM_TIMEOUT_SECONDS", "60") || "60";
  const evidenceRequired = settingValue(data, "LLM_EVIDENCE_REQUIRED", "true") || "true";
  const enabled = provider === "ollama";
  const configured = enabled && model !== "not selected";
  const evidenceOnlyMode = evidenceRequired.toLowerCase() !== "false";
  const healthStatus = !enabled ? "disabled" : configured ? "configured-local" : "needs-model";
  const enabledState = enabled ? "enabled in settings" : "disabled";
  const answerMode = !enabled
    ? "deterministic evidence"
    : configured
      ? "local LLM evidence-grounded with deterministic backup"
      : "unavailable until model is selected";
  const routingState = !enabled
    ? "provider none; no model route"
    : configured
      ? "ollama route configured; runtime health checked only during evidence-answer generation"
      : "ollama selected but no model configured";
  const fallbackState = evidenceOnlyMode
    ? "deterministic evidence fallback active"
    : "deterministic fallback still available; evidence requirement setting is false";
  const externalUse = "not used by default";
  const healthDetail = !enabled
    ? "No model calls are made while LLM_PROVIDER is none. Assistant uses deterministic evidence answers and insufficient-evidence responses."
    : configured
      ? "Settings does not contact Ollama. Evidence answers perform a timeout-bound local call only when retrieved evidence exists, then use a deterministic backup answer if unavailable."
      : "Select a local Ollama model before enabling evidence-grounded local generation. No token or cloud endpoint is required.";
  const guidance = !enabled
    ? "Local model generation is disabled; use deterministic evidence answers."
    : configured
      ? "Local Ollama routing is configured, but availability is verified only when an evidence-answer request runs."
      : "Set a local Ollama model and verify locally before expecting model-drafted answers.";
  const limitations = [
    "Evidence boundary: local generation is evidence-grounded and should not be treated as hidden memory or unsupported reasoning.",
    "Fallback behavior: deterministic evidence answers remain available when the provider is disabled, unavailable, or missing evidence.",
    "No installation: this UI does not install models, pull model files, or edit .env without the Settings dry-run/save flow.",
    "No hosted calls: IGY6 does not call hosted AI by default and this panel does not transfer source data."
  ];
  return {
    provider,
    baseUrl,
    model,
    timeout,
    evidenceRequired,
    enabledState,
    answerMode,
    routingState,
    fallbackState,
    externalUse,
    healthStatus,
    healthDetail,
    guidance,
    limitations,
    rawDiagnostics: {
      provider,
      ollama_base_url: baseUrl,
      model,
      timeout_seconds: timeout,
      evidence_required: evidenceRequired,
      enabled_state: enabledState,
      routing_state: routingState,
      fallback_state: fallbackState,
      answer_mode: answerMode,
      external_model_default: "blocked",
      hosted_ai_default: "not_used",
      secrets_required: "false"
    }
  };
}

export function settingValue(data: EnvSettingsResponse, key: string, fallback: string): string {
  const setting = data.settings.find((item) => item.key === key);
  if (!setting) return fallback;
  if (setting.secret) return setting.masked_value ?? fallback;
  return setting.value ?? fallback;
}

export function redactLlmUrl(value: string): string {
  if (value.includes("@")) return "http://[redacted]";
  return value;
}


export type { LlmDisplay };
