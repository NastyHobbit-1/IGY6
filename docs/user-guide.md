# User Guide

Open the local web UI:

```text
http://127.0.0.1:3000
```

IGY6 is organized around seven workflows:

- Home
- Assistant
- Data & Knowledge
- Work & Processing
- Reports
- Safety & Audit
- Settings

The old developer-console split between Chat, Agent Command, Sources, Evidence,
Memory, Work Queue, Approvals, Audit, Reports, and Settings has been folded into
these workflow sections. Advanced IDs, raw JSON, approval IDs, and route/debug
details are still present under Advanced panels.

## Normal PC User Examples

- Upload warranty text and ask when the warranty expires.
- Upload router or internet troubleshooting notes and ask what changed.
- Upload a folder inventory/export and ask what files look duplicated.
- Create a summary report from notes without sending anything externally.
- Ask Assistant: `What did I upload today?`
- Ask Assistant: `What does this document say about my bill?`
- Request approval before a system-changing action.

## Coder Examples

- Upload a build log and ask for the likely failure cause with evidence.
- Upload a repo status report and ask for the next DIFF recommendation.
- Use Assistant to show git status.
- Use Assistant to show latest DIFF.
- Create a work item for code review or route parity follow-up.
- Review audit events after an agent action.
- Inspect chunks/evidence created from a technical document.
- Render a migration or verification summary in Reports.

## Manual Upload Flow

1. Data & Knowledge -> Sources: create or select a `manual_upload` source.
2. Check source permission and approval status.
3. Safety & Audit -> Approvals: request approval if required.
4. Data & Knowledge -> Uploads & Collection: upload UTF-8 text.
5. Work & Processing: check collection and work status.
6. Data & Knowledge -> Evidence: inspect created documents, chunks, evidence,
   and source trails.
7. Assistant: ask a question over local evidence.

Field examples:

- Source name: `Router Troubleshooting Notes` or `IGY6 Build Logs`
- Source type: `manual_upload`
- Location: `local notes folder` or `local repo logs`
- Sensitivity: `private` or `internal`
- Allowed operations: `read, collect` or `read, collect, dry_run`
- Approval reason: `Allow IGY6 to process this uploaded troubleshooting note.`
- Coder approval reason: `Approve processing this local build log for evidence extraction.`
- Report render reason: `Create a summary of this uploaded bill.` or
  `Render a route parity verification summary.`

## Safety Notes

IGY6 is local-first and evidence-only by default. It does not claim Rust-only
operation while the manifest still requires FastAPI fallback. It does not send
evidence to an external model by default. It does not run arbitrary shell text
from Assistant input.

System-changing actions must clearly show approval requirements. Stack
start/stop/recovery actions require approval and fixed allowlisted runtime
capability.
