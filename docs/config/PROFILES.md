# IGY6 Configuration Profiles (grok branch)

These profiles apply environment defaults to `.env` using `scripts/bootstrap-profile.sh`. They are idempotent and can be re-applied safely. After changing profiles, restart the IGY6 stack.

How to apply:

```bash
scripts/bootstrap-profile.sh --wizard   # interactive
scripts/bootstrap-profile.sh quick-start
scripts/bootstrap-profile.sh --check    # readiness summary (no changes)
```

Keys affected:
- `EXTERNAL_MODEL_POLICY_DEFAULT` — default external model policy (blocked by default)
- `APPROVAL_REQUIRED_DEFAULT` — default for approval-required workflows
- `SINGLE_USER_MODE` — local single-user mode toggle
- `NEXT_TELEMETRY_DISABLED` — disables Next.js telemetry in the web image

Profiles:

- Quick Start (recommended)
  - Purpose: safest defaults for single-user local use
  - Keys: `EXTERNAL_MODEL_POLICY_DEFAULT=blocked`, `APPROVAL_REQUIRED_DEFAULT=true`, `SINGLE_USER_MODE=true`, `NEXT_TELEMETRY_DISABLED=1`
  - Restart required: yes (stack)

- Standard
  - Purpose: single-user defaults with explicit approvals
  - Keys: same as Quick Start
  - Restart required: yes

- Advanced
  - Purpose: single-user with commented advanced knobs ready to enable
  - Keys: same as Quick Start; commented examples for vector size/LLM timeout
  - Restart required: yes (and after uncommenting advanced keys)

- Expert
  - Purpose: multi-user capable posture under owner control
  - Keys: `EXTERNAL_MODEL_POLICY_DEFAULT=blocked`, `APPROVAL_REQUIRED_DEFAULT=true`, `SINGLE_USER_MODE=false`, `NEXT_TELEMETRY_DISABLED=1`
  - Restart required: yes

Effective behavior:
- These keys are validated and surfaced by the Settings → Environment panel and used by the runtime policy posture (e.g., approvals remain required by default; external model use is blocked by default; single-user UI behavior stays enabled/disabled per profile).

