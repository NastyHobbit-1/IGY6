# IGY6 Environment Reference (selected keys)

This reference lists the configuration keys exposed by the installer profiles and related UI.

Notes:
- Where marked, changes require restarting the IGY6 stack to take effect.
- Defaults shown reflect the Quick Start profile on the `grok` branch.

Keys:

- `EXTERNAL_MODEL_POLICY_DEFAULT`
  - Purpose: default rule for external/hosted model use
  - Default: `blocked`
  - Allowed values: `blocked`, `metadata_only`, `allowed_with_approval`
  - Restart required: yes

- `APPROVAL_REQUIRED_DEFAULT`
  - Purpose: default whether sensitive workflows require approval
  - Default: `true`
  - Allowed values: `true` / `false`
  - Restart required: yes

- `SINGLE_USER_MODE`
  - Purpose: local single-user mode switch for UI/runtime posture
  - Default: `true`
  - Allowed values: `true` / `false`
  - Restart required: yes

- `NEXT_TELEMETRY_DISABLED`
  - Purpose: disable Next.js telemetry in the web image
  - Default: `1`
  - Allowed values: `0` / `1`
  - Restart required: yes

Health/readiness:

- Required for building CLI: `cargo`
- Optional for running now: `docker` (Docker Desktop + Compose v2 recommended)
- `scripts/bootstrap-profile.sh --check` prints: `.env` present/missing, cargo/docker presence, and available profile count.

