# Local LLM Provider Plan

IGY6 may add optional local LLM support in later DIFFs. This plan defines the
provider contract and safety posture before normal Assistant answer generation
uses model-calling code.

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

## Implementation Status

- DIFF-126 documented the provider plan and safe disabled-by-default config.
- DIFF-127 adds the Rust `igy6-llm` crate with disabled-provider handling, a
  local-Ollama-only adapter, safe health/generate helpers, timeout propagation,
  local URL validation, structured output, explicit errors, output preview
  redaction, and fake-transport tests.
- DIFF-127 does not wire local LLM generation into Assistant and does not call a
  real Ollama process during tests.
- DIFF-128 wires optional local LLM generation into evidence answer generation
  behind evidence-required checks. Provider disabled, unavailable, invalid, or
  timed out states fall back to deterministic evidence answers. Missing evidence
  returns insufficient evidence without calling the provider. Tests use fake
  transports only.
- DIFF-129 adds user-facing Assistant and Settings status for provider, local
  Ollama URL, model, health/status text, timeout, evidence-required state,
  answer mode, examples, and Advanced raw provider diagnostics without requiring
  secrets.
- DIFF-130 adds safe local Ollama setup support and task-based routing config in
  `configs/local-llm-routing.json`. The setup helper is check-only by default,
  pulls only the approved default models when explicitly requested, and can
  safely write local `.env` Ollama keys after creating a backup.
- DIFF-131 connects the routing config to Rust evidence-answer generation. When
  local Ollama generation is enabled, Rust loads and validates the routing file,
  selects the requested task route, and sends the route model, system
  instruction, and temperature to the local provider. Unknown tasks use
  `chat_default`; disabled, unavailable, invalid, timed-out, or insufficient
  evidence paths still use deterministic fallback.

## Task Routing

Task routing maps local work types to a model, system instruction, temperature,
purpose, and context note. All routes require evidence.

| Task | Model | Purpose |
| --- | --- | --- |
| `code_repo` | `qwen2.5-coder:7b` | Code, logs, stack traces, DIFF prompts, repo summaries, route parity, scripts |
| `evidence_summary` | `llama3.1:8b` | Uploaded notes, warranty text, bills, router notes, general evidence records |
| `fast_triage` | `gemma3:4b` | Quick classification, short summaries, first-pass triage, UI helper explanations |
| `report_draft` | `llama3.1:8b`; optional `gemma3:12b` | Longer evidence-grounded reports and migration summaries |
| `action_explanation` | `gemma3:4b` | Approval-required action explanations, safety reasons, audit summaries |
| `chat_default` | `llama3.1:8b` | Default Assistant evidence-grounded chat when not code-specific |

`qwen2.5-coder:7b` is the default for code/repo tasks because it is tuned for
code, logs, scripts, and repository summaries while staying practical for an RTX
3060 12GB class machine. `llama3.1:8b` is the default general evidence model.
`gemma3:4b` is the fast/lightweight route. `gemma3:12b` is optional and heavier.

Rust evidence-answer requests may include `task_name` or `task`. Supported
values are the route names above. If the value is missing or unknown, Rust uses
`chat_default`. `LLM_PROVIDER=none` still means no model call.

## Ollama Setup Commands

Check local state:

```bash
scripts/ollama-local-setup.sh --check
scripts/ollama-local-setup.sh --list-recommended
ollama --version
curl http://127.0.0.1:11434/api/tags
```

Install Ollama manually:

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

Pull default models:

```bash
ollama pull qwen2.5-coder:7b
ollama pull llama3.1:8b
ollama pull gemma3:4b
```

Test a model manually:

```bash
ollama run qwen2.5-coder:7b
```

Configure IGY6:

```env
LLM_PROVIDER=ollama
OLLAMA_BASE_URL=http://host.docker.internal:11434
OLLAMA_MODEL=qwen2.5-coder:7b
LLM_EVIDENCE_REQUIRED=true
```

Return to deterministic mode:

```env
LLM_PROVIDER=none
```

## Not Implemented In DIFF-126 Through DIFF-131

These DIFFs do not add external model providers, LLM action execution, broad
retrieval rewrites, mandatory Ollama startup, or Rust-only operation.
