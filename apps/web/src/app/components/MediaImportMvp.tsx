import { MEDIA_IMPORT_TYPES } from "./constants";
import { formatBytes } from "./helpers";
import { ClientScript, DomJsonScript } from "@/lib/use-dom-script";

export function MediaImportMvp() {
  const browserApiBaseUrl = process.env.NEXT_PUBLIC_API_BASE_URL ?? "http://127.0.0.1:8000";
  const mediaTypesJson = JSON.stringify(MEDIA_IMPORT_TYPES).replace(/</g, "\\u003c");
  const script = `
(() => {
  const root = document.querySelector("[data-media-import-mvp]");
  if (!root) return;
  const mediaTypes = JSON.parse(root.querySelector("[data-media-import-types-json]")?.textContent || "[]");
  const form = root.querySelector("[data-media-import-preview-form]");
  const typeSelect = root.querySelector("[name='media_type']");
  const fileInput = root.querySelector("[name='media_file']");
  const result = root.querySelector("[data-media-import-result]");
  const typeStatus = root.querySelector("[data-media-import-type-status]");
  const value = (name) => root.querySelector("[name='" + name + "']")?.value?.trim() || "";
  const selectedType = () => mediaTypes.find((item) => item.key === typeSelect?.value) || mediaTypes[0];
  const formatBytes = (size) => {
    if (!Number.isFinite(size)) return "unknown";
    if (size < 1024) return size + " B";
    if (size < 1024 * 1024) return (size / 1024).toFixed(1) + " KB";
    return (size / (1024 * 1024)).toFixed(1) + " MB";
  };
  const render = (payload) => {
    if (!result) return;
    result.innerHTML = "";
    const title = document.createElement("strong");
    title.textContent = payload.title;
    const body = document.createElement("span");
    body.textContent = payload.message;
    result.append(title, body);
    const details = document.createElement("dl");
    payload.details.forEach((detail) => {
      const term = document.createElement("dt");
      term.textContent = detail.label;
      const description = document.createElement("dd");
      description.textContent = detail.value;
      details.append(term, description);
    });
    result.append(details);
    const list = document.createElement("ul");
    payload.next.forEach((step) => {
      const item = document.createElement("li");
      item.textContent = step;
      list.appendChild(item);
    });
    result.append(list);
  };
  const updateStatus = () => {
    const type = selectedType();
    if (!typeStatus || !type) return;
    typeStatus.textContent = type.label + " status: " + type.status + ". " + type.unsupportedReason;
  };
  typeSelect?.addEventListener("change", updateStatus);
  updateStatus();
  const postJson = async (path, body) => {
    const apiBaseUrl = root.getAttribute("data-api-base-url");
    const response = await fetch(apiBaseUrl + path, { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
    const payload = await response.json().catch(() => ({}));
    if (!response.ok) throw new Error(JSON.stringify(payload));
    return payload;
  };
  const textToBase64 = (text) => {
    const bytes = new TextEncoder().encode(text);
    let binary = "";
    bytes.forEach((byte) => { binary += String.fromCharCode(byte); });
    return btoa(binary);
  };
  root.querySelector("[data-media-collect-text]")?.addEventListener("click", async () => {
    const button = root.querySelector("[data-media-collect-text]");
    const extractedText = value("media_extracted_text");
    const label = value("media_label") || "media-extract.txt";
    if (!extractedText) {
      render({ title: "Text required", message: "Paste reviewed extracted text before collecting.", details: [], next: ["OCR/transcription is not run here. Paste reviewed UTF-8 text only."] });
      return;
    }
    if (button) { button.disabled = true; button.textContent = "Collecting..."; }
    try {
      const created = await postJson("/sources", {
        name: label.slice(0, 80),
        source_type: "media_file",
        location: label,
        sensitivity: "internal",
        metadata_json: { created_from: "media_import_panel" },
        permission: {
          scope_json: { media_label: label, media_type: selectedType().key },
          allowed_operations: ["dry_run", "read", "collect"],
          external_model_policy: "blocked",
          approval_required: false
        }
      });
      const permission = (created.permissions || []).find((item) => (item.allowed_operations || []).includes("collect"));
      const upload = await postJson("/collection-runs/manual-upload", {
        source_id: created.id,
        source_permission_id: permission.id,
        filename: label.endsWith(".txt") ? label : label + ".txt",
        mime_type: "text/plain",
        content_base64: textToBase64(extractedText),
        metadata_json: { submitted_from: "media_import_panel", media_type: selectedType().key },
        requested_by_actor_id: "local-owner"
      });
      render({
        title: "Media text collected",
        message: "Reviewed extracted text was stored locally and normalization work was queued.",
        details: [
          { label: "source", value: created.id },
          { label: "collection run", value: upload?.id || "not returned" },
          { label: "work item", value: upload?.summary_json?.normalization_work_item_id || "not returned" }
        ],
        next: ["Open Work to watch processing.", "Use Deep scan or Media Library for binary image/video artifacts."]
      });
    } catch (error) {
      render({ title: "Collection failed", message: error instanceof Error ? error.message : "Unknown error", details: [], next: [] });
    } finally {
      if (button) { button.disabled = false; button.textContent = "Collect extracted text"; }
    }
  });
  form?.addEventListener("submit", (event) => {
    event.preventDefault();
    const type = selectedType();
    const file = fileInput?.files?.[0] || null;
    const extractedText = value("media_extracted_text");
    const fileSize = file ? file.size : null;
    const bounded = fileSize === null || fileSize <= 25 * 1024 * 1024;
    render({
      title: "Media import preview",
      message: extractedText ? "Reviewed extracted text can be collected with the button below. Binary parsing/OCR/transcription are not run in this panel." : "No binary media was uploaded, parsed, OCRed, transcribed, or sent to a hosted service.",
      details: [
        { label: "media type", value: type.label + " · " + type.status },
        { label: "file label", value: file ? file.name : (value("media_label") || "not selected") },
        { label: "browser-reported MIME", value: file?.type || "not provided" },
        { label: "size", value: file ? formatBytes(file.size) : "not selected" },
        { label: "size bound", value: bounded ? "within 25 MB preview bound" : "too large for this MVP preview" },
        { label: "accepted input", value: type.acceptedInput },
        { label: "extraction status", value: extractedText ? "user-provided extracted text can be collected through Guided Upload after review" : type.unsupportedReason },
        { label: "lineage posture", value: "future implementation must preserve source, artifact, document, chunk, and evidence lineage" },
        { label: "external services", value: "none" }
      ],
      next: [
        type.safeNext,
        "Do not paste secrets, private paths, credentials, or unreviewed media contents.",
        "This panel records no artifact; use Guided Upload only for reviewed UTF-8 text."
      ]
    });
  });
})();
`;

  return (
    <section className="guidedManualText" id="media-import" data-media-import-mvp data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>PDF, image, audio, and video import.</strong>
        <span>Preview media posture here. Collect reviewed extracted text locally; use Deep scan / Media Library for binary image and video artifacts.</span>
      </div>
      <form className="guidedManualForm" data-media-import-preview-form>
        <label>
          <span>Media type</span>
          <select name="media_type" defaultValue="pdf">
            {MEDIA_IMPORT_TYPES.map((type) => (
              <option key={type.key} value={type.key}>{type.label}</option>
            ))}
          </select>
        </label>
        <p className="actionHint" data-media-import-type-status />
        <label>
          <span>File label if no file selected</span>
          <input name="media_label" placeholder="statement.pdf, screenshot.png, meeting-audio.wav" />
        </label>
        <label>
          <span>Optional local file metadata preview</span>
          <input name="media_file" type="file" accept=".pdf,image/*,audio/*,video/*" />
        </label>
        <label>
          <span>Reviewed extracted text or transcript if already available</span>
          <textarea name="media_extracted_text" rows={5} placeholder="Paste reviewed extracted text or transcript to collect locally." />
        </label>
        <div className="guidedManualActions">
          <button type="submit">Preview media import status</button>
          <button type="button" data-media-collect-text>Collect extracted text</button>
          <span>Binary parsing/OCR/transcription are not run here. Deep scan collects binary media artifacts.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-media-import-result>
        <strong>Ready</strong>
        <span>Select a media type and preview support status, size bounds, and safe next steps.</span>
      </div>
      <DomJsonScript marker="data-media-import-types-json" json={mediaTypesJson} />
      <ClientScript script={script} />
    </section>
  );
}

