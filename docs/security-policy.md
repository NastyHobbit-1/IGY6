# Security Policy (grok branch)

On this branch the program is password protected (default "ThatDog123") with easy password changing in the User & Security UI section. Optional TOTP authenticator support (any standard app) is off by default until explicitly linked.

## Core Posture (grok branch)

- Local-first and local-only for all collected content.
- Deep/thorough collector can reach any target the process has access to (local FS, web, system, WiFi, etc.) and stores **everything only inside the instance** (full-res images/videos fetched directly from their original sources, complete info, full provenance).
- No content exfiltration. The only outbound activity is the fetches the operator explicitly triggers.
- Dynamic clear local URLs (auto-switches to a free port and reports the usable address).
- Password + optional TOTP gate for powerful/protected features (collector, etc.).
- Full audit trail and evidence/graph ties for everything collected.

## Password & Authenticator

- Default password "ThatDog123".
- Change password anytime via the User section (requires current password; TOTP code if enabled).
- TOTP (standard RFC 6238) is off by default. Generate secret/otpauth URL (any authenticator app works — Google Authenticator, Authy, etc.), add it, then confirm a code to enable. Once on, protected actions also require a current code.
- Status and linking are exposed via /user/status, /user/generate-totp, /user/confirm-totp, /user/change-password (all require current credentials).

## Collection & Media

- Aggressive deep collection is a deliberate feature on this branch.
- Images and videos are stored at full/original resolution from the source and viewable in the dedicated Media Library at full fidelity.
- All data (artifacts with mime/kind, evidence, graph, audit) stays local.

## Operational Security

- Start with scripts/run.sh after preparing your data root.
- Unlock the UI with current password (and TOTP code when enabled).
- Use the collector responsibly — it will ingest anything reachable.
- Manage password and optional authenticator in the User & Security section.
- Keep your IGY6_DATA_ROOT on trusted storage with proper permissions.

All documentation has been updated to this program-only, operating view for the grok branch. The program is self-contained, auditable, and password (optionally TOTP) protected while delivering powerful local collection and media capabilities.
- Worker-side execution only after approval.

Sensitive actions include file writes, PC setting changes, router changes,
account actions, repository writes, website changes, sensitive data export, and
external model use with sensitive content.
