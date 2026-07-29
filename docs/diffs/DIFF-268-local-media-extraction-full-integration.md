# DIFF-268 - Local media extraction full integration

Status: Complete (code + install + UI + crate wiring)

## Branch

- Branch: `grok` only
- Does not modify scraping, bypass, max-reach, session fetch, or full-access collection paths

## Purpose

Make PDF / image / audio / video extraction part of the product:

1. Install local tools at first install and in the worker image (pdftotext, tesseract, ffmpeg, whisper).
2. Extract text using those tools on this machine.
3. Store original binary + extracted text only inside IGY6.
4. UI uploads real media binaries (not paste-only).

## User rules applied

- Maximize gathering from available sources (media upload path added; existing scrape/bypass untouched).
- Local and external *tools* may run for interpretation; **processed results stay inside IGY6**.
- No fabricated product restrictions beyond what the owner ordered.

## Implementation

### New crate: `igy6-media-extract`

- `extract_text_from_media` / `extract_or_utf8`
- PDF → `pdftotext`
- Image → `tesseract`
- Audio/video → `ffmpeg` to WAV → local `whisper`
- Tool availability probe
- Unit tests for classification + UTF-8 path

### Normalization

- `normalize_artifact_bytes` and `normalize_raw_artifact` route media through extraction
- Extraction method/tool/success recorded on document metadata
- Plain text path unchanged

### Worker / install

- Worker depends on `igy6-media-extract`
- Worker Dockerfile installs: poppler-utils, tesseract-ocr (+ eng), ffmpeg, openai-whisper
- `install.sh` / `install.ps1` install host-side tools when possible and instruct worker rebuild

### UI

- `MediaImportMvp` uploads binary via existing `/collection-runs/manual-upload` with correct MIME
- Creates `media_file` source
- Shows work-item guidance

## Unchanged (verified intact)

- Deep Fetch / max-reach scripts and UI
- Auto bypass / Session Fetch / Public fetch
- `bypass_intel.rs` and BypassIntelPanel
- Host bridge ensure-max-reach
- Full-access collection route

## Owner steps after pull

```bash
git checkout grok && git pull
# rebuild worker so tools are in the image
docker compose -f infra/docker-compose.yml build worker
docker compose -f infra/docker-compose.yml up -d worker
# optional host tools
./install.sh   # or install.ps1 on Windows
```

## Verification

- Scraping/bypass paths not edited in this DIFF
- New crate + normalization tests included
- First worker image build is longer (apt + whisper install)

## Notes

- Processed transcripts/OCR text are stored as normalized documents/evidence inside IGY6 only.
- Quality depends on local engines (Tesseract / Whisper), not cloud services.
- Rebuild worker image required for tools to be present inside the container.
