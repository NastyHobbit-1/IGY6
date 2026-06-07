# DIFF-237 - PDF / Image / Audio / Video Import MVP

Status: Complete

## Branch And Baseline

- Current branch before work: `dev`
- HEAD before work: `14e8bfc Complete DIFF-236 browser web router collector MVP`
- `dev` tracking state before work: ahead of `origin/dev` by 2 commits
- Working tree before work: clean

## Purpose

Add a safe media import foundation for PDF, image, audio, and video while
keeping unsupported parsing states honest.

## Files Inspected

- `docs/diffs/DIFF-235-source-expansion-connector-contract-foundation.md`
- `docs/diffs/DIFF-236-browser-web-router-collector-mvp.md`
- `docs/plans/IGY6_COMPLETION_BUILD_PLAN_DIFF_210_ONWARD.md`
- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- Existing manual upload, artifact, normalization, chunking, evidence, source,
  permission, and collection run UI flows inspected during the batch pre-work

## Implementation

- Added a PDF / Image / Audio / Video Import Foundation panel in Add Data.
- Added media support status entries:
  - PDF: metadata preview and user-provided extracted text only
  - Image/screenshot: unsupported/planned OCR
  - Audio: unsupported/planned transcription
  - Video: unsupported/planned transcription/frame OCR
- Added local browser-side preview of selected file name, MIME type, and size.
- Added a 25 MB preview-bound posture message.
- Added safe next-step guidance to use Guided Upload only for reviewed UTF-8
  extracted text or transcripts.
- Updated the UI guide with media import behavior and unsupported parsing
  limits.

## Scope Confirmation

This DIFF does not upload, parse, OCR, transcribe, normalize, chunk, or create
artifacts from binary media. It adds a product-facing metadata/status preview
and keeps extraction limits explicit.

No hosted OCR/transcription/AI API call, hidden external data transfer,
unbounded binary processing, raw media dump, private path dump, credential
collection, `.env` edit, runtime/private data dump, backend route, persistence
schema, or worker behavior was added.

## Files Changed

- `apps/web/src/app/page.tsx`
- `docs/ui/README.md`
- `docs/diffs/DIFF-237-pdf-image-audio-video-import-mvp.md`

## Verification Commands And Results

Passed:

- `git status --short`
- `git diff --check`
- `git diff --name-status`
- `npm --prefix apps/web run build`
- `git ls-files AGENTS.md .codex Adaptive_Intelligence_System_Coder_Build_Instructions_v2.md docs/agents docs/plans | sort`
- `grep -R "Status: Active\|Status: In Progress\|Status: Draft" docs/diffs 2>/dev/null || true`

Not run:

- Rust checks were not required because no Rust files changed.
- Script syntax checks were not required because no scripts changed.
- Full Docker smoke was not run from Codex per owner instruction.

## Verification Summary

- Next.js production build passed.
- Working-tree whitespace check passed.
- Private/dev files remained tracked on `dev`.
- Stale status scan still reports older out-of-scope draft/status strings in
  historical DIFF records and command examples; this DIFF is
  `Status: Complete`.
