# Local LLM Provider Plan

IGY6 may add optional local LLM support in later DIFFs. This plan defines the
provider contract and safety posture before any model-calling code is added.

## Current Posture

- LLM use is optional and disabled by default.
- The starting provider is Ollama because it can run locally on the user's
  machine.
- IGY6 must not call external model services by default.
- Evidence-first behavior remains the system contract.
- Deterministic evidence answers remain the fallback when the provider is
  disabled, unavailable, timed out, or not safely configured.
- When evidence is required and no retrieved evidence exists, the answer must
  say there is insufficient evidence.
- LLM answers must cite retrieved evidence or source trails where possible.
- LLM prompts must be bounded to the retrieved evidence packet and necessary
  task instructions, not broad runtime/private data.

## Provider Interface Requirements

Any provider adapter must expose:

- Provider name, such as `none` or `ollama`.
- Base URL, such as `http://host.docker.internal:11434` for Docker-to-host
  Ollama on many desktop setups.
- Model name, such as a locally installed Ollama model.
- Health check that verifies the configured local provider is reachable without
  generating an answer.
- Generate endpoint that accepts a bounded prompt and returns text plus provider
  status metadata.
- Timeout in seconds.
- Maximum context/evidence budget before prompt construction.
- Redaction and safety behavior for logs, errors, and UI status.
- No secrets in logs, audit metadata, prompt traces, or raw response panels.

Provider adapters must fail closed. A missing provider, unsupported provider,
invalid URL, timeout, or unsafe response must return a clear unavailable or
insufficient-evidence status and fall back to deterministic evidence behavior.

## Configuration Keys

Safe local defaults are documented in `.env.example`:

```env
LLM_PROVIDER=none
OLLAMA_BASE_URL=http://host.docker.internal:11434
OLLAMA_MODEL=
LLM_TIMEOUT_SECONDS=60
LLM_EVIDENCE_REQUIRED=true
```

`LLM_PROVIDER=none` means no model calls. `LLM_PROVIDER=ollama` may be used by a
later implementation DIFF to enable a local Ollama adapter. `OLLAMA_MODEL` is
left empty until the user intentionally chooses a locally installed model.

These keys must not contain tokens, API keys, private keys, or cloud model
secrets.

## Evidence-Grounded Answer Rules

Later answer-generation work must follow these rules:

- Retrieve evidence first.
- Build a bounded evidence packet with source trails and citation handles.
- Redact secret-like values before logging or status display.
- Call the local provider only when enabled and healthy.
- Require citations when making factual claims from evidence.
- Do not invent unsupported claims.
- Do not execute actions through the LLM.
- Do not send evidence to external services by default.
- Return deterministic evidence output if the provider is disabled or fails.
- Return insufficient-evidence status if evidence is required but absent.

## User-Facing Behavior

Settings should eventually show:

- Provider status: disabled, local provider configured, unavailable, or healthy.
- Provider name and redacted base URL.
- Model name if configured.
- Evidence-required state.
- Timeout value.
- Clear copy that local LLM support is optional and does not change approval
  requirements for actions.

Assistant should eventually show:

- Whether the answer came from deterministic evidence mode or local LLM
  evidence-grounded mode.
- Citations/source trails.
- Insufficient-evidence messages when no supporting evidence exists.
- Provider unavailable messages without exposing private config details.

## Example Use Cases

Normal PC users:

- Upload a warranty note and ask when the warranty expires.
- Upload router troubleshooting notes and ask what changed.
- Create a summary report from local notes without sending the notes externally.

Seasoned coders:

- Upload a build log and ask for the likely failure cause with evidence.
- Upload a route parity verification summary and ask for the next DIFF
  recommendation.
- Ask for a concise migration summary with citations to uploaded evidence.

## Not Implemented In DIFF-126

DIFF-126 does not add provider code, call Ollama, wire Assistant answers to an
LLM, or change backend behavior. Those changes require later DIFFs with focused
tests.
