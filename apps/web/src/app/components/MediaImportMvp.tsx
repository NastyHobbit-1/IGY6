import { MEDIA_IMPORT_TYPES } from "./constants";
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
    (payload.details || []).forEach((detail) => {
      const term = document.createElement("dt");
      term.textContent = detail.label;
      const description = document.createElement("dd");
      description.textContent = detail.value;
      details.append(term, description);
    });
    result.append(details);
    const list = document.createElement("ul");
    (payload.next || []).forEach((step) => {
      const item = document.createElement("li");
      item.textContent = step;
      list.appendChild(item);
    });
    result.append(list);
  };
  const updateStatus = () => {
    const type = selectedType();
    if (!typeStatus || !type) return;
    typeStatus.textContent = type.label + " — " + type.status + ". " + (type.acceptedInput || "");
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
  const fileToBase64 = (file) => new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const text = String(reader.result || "");
      const comma = text.indexOf(",");
      resolve(comma >= 0 ? text.slice(comma + 1) : text);
    };
    reader.onerror = () => reject(new Error("Failed to read file"));
    reader.readAsDataURL(file);
  });
  const defaultMime = (typeKey, file) => {
    if (file?.type) return file.type;
    if (typeKey === "pdf") return "application/pdf";
    if (typeKey === "image") return "image/png";
    if (typeKey === "audio") return "audio/wav";
    if (typeKey === "video") return "video/mp4";
    return "application/octet-stream";
  };
  root.querySelector("[data-media-upload-binary]")?.addEventListener("click", async () => {
    const button = root.querySelector("[data-media-upload-binary]");
    const type = selectedType();
    const file = fileInput?.files?.[0] || null;
    if (!file) {
      render({ title: "File required", message: "Choose a PDF, image, audio, or video file to upload.", details: [], next: ["Binary is stored locally; extraction runs in the worker with installed tools."] });
      return;
    }
    if (file.size > 200 * 1024 * 1024) {
      render({ title: "File too large", message: "Current bound is 200 MB for media upload.", details: [{ label: "size", value: formatBytes(file.size) }], next: [] });
      return;
    }
    if (button) { button.disabled = true; button.textContent = "Uploading..."; }
    try {
      const contentBase64 = await fileToBase64(file);
      const label = value("media_label") || file.name || "media-upload";
      const created = await postJson("/sources", {
        name: label.slice(0, 80),
        source_type: "media_file",
        location: file.name,
        sensitivity: "internal",
        metadata_json: { created_from: "media_import_panel", media_type: type.key },
        permission: {
          scope_json: { media_label: label, media_type: type.key },
          allowed_operations: ["dry_run", "read", "collect", "normalize", "extract_metadata"],
          external_model_policy: "blocked",
          approval_required: false
        }
      });
      const permission = (created.permissions || []).find((item) => (item.allowed_operations || []).includes("collect"));
      if (!permission?.id) throw new Error("No collect permission on created media source");
      const upload = await postJson("/collection-runs/manual-upload", {
        source_id: created.id,
        source_permission_id: permission.id,
        filename: file.name,
        mime_type: defaultMime(type.key, file),
        content_base64: contentBase64,
        metadata_json: {
          submitted_from: "media_import_panel",
          media_type: type.key,
          original_filename: file.name,
          extract_pipeline: "local_tools"
        },
        requested_by_actor_id: "local-owner"
      });
      render({
        title: "Media uploaded",
        message: "Binary stored locally. Worker normalization extracts text with pdftotext / tesseract / ffmpeg+whisper. Processed text stays inside IGY6.",
        details: [
          { label: "source", value: created.id },
          { label: "collection run", value: upload?.id || "not returned" },
          { label: "work item", value: upload?.summary_json?.normalization_work_item_id || "not returned" },
          { label: "mime", value: defaultMime(type.key, file) },
          { label: "size", value: formatBytes(file.size) }
        ],
        next: [
          "Open Work to watch normalization / extraction.",
          "When complete, open Chat and ask over the extracted evidence.",
          "Original binary remains in the artifact store."
        ]
      });
    } catch (error) {
      render({ title: "Upload failed", message: error instanceof Error ? error.message : "Unknown error", details: [], next: ["Confirm the API is running and rebuild the worker image after install."] });
    } finally {
      if (button) { button.disabled = false; button.textContent = "Upload media file"; }
    }
  });
  form?.addEventListener("submit", (event) => {
    event.preventDefault();
    const type = selectedType();
    const file = fileInput?.files?.[0] || null;
    render({
      title: "Media import ready",
      message: "Select a file and click Upload media file. Extraction uses local tools installed with the product; results stay inside IGY6.",
      details: [
        { label: "media type", value: type.label + " · " + type.status },
        { label: "file", value: file ? file.name : (value("media_label") || "not selected") },
        { label: "MIME", value: file?.type || "not provided" },
        { label: "size", value: file ? formatBytes(file.size) : "not selected" },
        { label: "pipeline", value: "local pdftotext / tesseract / ffmpeg+whisper via worker" }
      ],
      next: [type.safeNext || "Upload the file to start extraction."]
    });
  });
})();
`;

  return (
    <section className="guidedManualText" id="media-import" data-media-import-mvp data-api-base-url={browserApiBaseUrl}>
      <div className="guidedManualNotice">
        <strong>PDF, image, audio, and video import.</strong>
        <span>
          Upload the binary. Local tools extract text (PDF text layer, OCR, transcription). Original media and extracted text stay inside this IGY6 instance.
        </span>
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
          <span>Optional label</span>
          <input name="media_label" placeholder="statement.pdf, screenshot.png, meeting-audio.wav" />
        </label>
        <label>
          <span>Media file</span>
          <input name="media_file" type="file" accept=".pdf,image/*,audio/*,video/*" />
        </label>
        <div className="guidedManualActions">
          <button type="submit">Preview media status</button>
          <button type="button" data-media-upload-binary>Upload media file</button>
          <span>Extraction runs in the worker with tools installed at product install / image build.</span>
        </div>
      </form>
      <div className="guidedManualResult" data-media-import-result>
        <strong>Ready</strong>
        <span>Choose a file and upload. Work tab shows processing; Chat uses extracted evidence when ready.</span>
      </div>
      <DomJsonScript marker="data-media-import-types-json" json={mediaTypesJson} />
      <ClientScript script={script} />
    </section>
  );
}
