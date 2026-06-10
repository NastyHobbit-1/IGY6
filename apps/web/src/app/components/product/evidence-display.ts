import type { EvidenceItemRecord, RawArtifactRecord, SourceRecord } from "../types";
import { excerpt, formatDate } from "../helpers";

export type ContentTypeLabel =
  | "Text"
  | "PDF"
  | "Image"
  | "Video"
  | "Audio"
  | "Website"
  | "Folder"
  | "Document"
  | "Unknown";

export type EvidenceCardModel = {
  id: string;
  contentType: ContentTypeLabel;
  title: string;
  summary: string;
  sourceLabel: string;
  statusLabel: string;
  statusState: string;
  dateLabel: string;
  whyItMatters: string;
  viewHref: string | null;
  technical: Record<string, unknown>;
};

const EXTENSION_LABELS: Record<string, ContentTypeLabel> = {
  txt: "Text",
  md: "Text",
  markdown: "Text",
  pdf: "PDF",
  png: "Image",
  jpg: "Image",
  jpeg: "Image",
  webp: "Image",
  gif: "Image",
  svg: "Image",
  mp4: "Video",
  webm: "Video",
  mov: "Video",
  mp3: "Audio",
  wav: "Audio",
  m4a: "Audio",
  html: "Website",
  htm: "Website",
};

function extensionFromPath(value: string | null | undefined): string | null {
  if (!value) return null;
  const clean = value.split("?")[0]?.split("#")[0] ?? value;
  const dot = clean.lastIndexOf(".");
  if (dot < 0) return null;
  return clean.slice(dot + 1).toLowerCase();
}

export function normalizeContentType(input: {
  mimeType?: string | null;
  fileName?: string | null;
  sourceType?: string | null;
  evidenceType?: string | null;
  url?: string | null;
}): ContentTypeLabel {
  const mime = (input.mimeType ?? "").toLowerCase().trim();
  const ext = extensionFromPath(input.fileName ?? input.url ?? null);
  if (mime.startsWith("image/")) return "Image";
  if (mime.startsWith("video/")) return "Video";
  if (mime.startsWith("audio/")) return "Audio";
  if (mime === "application/pdf" || ext === "pdf") return "PDF";
  if (mime.startsWith("text/") || ext === "txt" || ext === "md" || ext === "markdown") return "Text";
  if (mime === "text/html" || ext === "html" || ext === "htm") return "Website";
  if ((input.url ?? "").startsWith("http")) return "Website";
  if ((input.sourceType ?? "").includes("folder") || (input.sourceType ?? "").includes("project")) {
    return "Folder";
  }
  if (ext && EXTENSION_LABELS[ext]) return EXTENSION_LABELS[ext];
  if (input.evidenceType === "document_chunk") return "Document";
  if ((input.sourceType ?? "").includes("web")) return "Website";
  return "Unknown";
}

function readableTitle(item: EvidenceItemRecord, source?: SourceRecord): string {
  const statement = item.statement?.trim();
  if (statement && statement.length > 8 && !statement.startsWith("[")) {
    return statement.length > 96 ? `${statement.slice(0, 93)}…` : statement;
  }
  const meta = item.metadata_json ?? {};
  const originalUrl = typeof meta.original_url === "string" ? meta.original_url : null;
  const fileName = typeof meta.filename === "string" ? meta.filename : null;
  if (fileName) return fileName;
  if (originalUrl) {
    try {
      const parsed = new URL(originalUrl);
      const pathTail = parsed.pathname.split("/").filter(Boolean).pop();
      if (pathTail) return pathTail;
      return parsed.hostname;
    } catch {
      return originalUrl.length > 72 ? `${originalUrl.slice(0, 69)}…` : originalUrl;
    }
  }
  if (source?.name) return `From ${source.name}`;
  return "Saved item";
}

export function buildEvidenceCardModel(
  item: EvidenceItemRecord,
  source?: SourceRecord,
  artifact?: RawArtifactRecord,
): EvidenceCardModel {
  const meta = item.metadata_json ?? {};
  const mimeType = artifact?.mime_type ?? (typeof meta.mime_type === "string" ? meta.mime_type : null);
  const fileName =
    (typeof meta.filename === "string" ? meta.filename : null)
    ?? (typeof meta.original_url === "string" ? meta.original_url : null);
  const contentType = normalizeContentType({
    mimeType,
    fileName,
    sourceType: source?.source_type,
    evidenceType: item.evidence_type,
    url: typeof meta.original_url === "string" ? meta.original_url : null,
  });

  const artifactId = artifact?.id ?? item.document_id ?? null;
  const viewHref =
    contentType === "Image" && artifactId ? `/api/artifacts/${artifactId}/content` : null;

  return {
    id: item.id,
    contentType,
    title: readableTitle(item, source),
    summary: excerpt(item.statement, 180),
    sourceLabel: source?.name ?? source?.source_type?.replace(/_/g, " ") ?? "Local source",
    statusLabel: item.confidence && item.confidence > 0.7 ? "High confidence" : "Saved locally",
    statusState: item.confidence && item.confidence > 0.7 ? "high" : "recorded",
    dateLabel: formatDate(item.created_at),
    whyItMatters:
      item.evidence_type === "document_chunk"
        ? "This excerpt can support answers and summaries in chat."
        : "This record may be cited when you ask IGY6 questions.",
    viewHref,
    technical: {
      evidence_id: item.id,
      document_id: item.document_id,
      chunk_id: item.chunk_id,
      evidence_type: item.evidence_type,
      mime_type: mimeType,
      artifact_id: artifact?.id ?? null,
      content_hash: artifact?.content_hash ?? null,
      metadata_json: meta,
    },
  };
}

export function isImageArtifact(artifact: RawArtifactRecord): boolean {
  const mime = (artifact.mime_type ?? "").toLowerCase();
  if (mime.startsWith("image/")) return true;
  const meta = artifact.metadata_json ?? {};
  const kind = String(meta.kind ?? "").toLowerCase();
  return kind === "image";
}

export function galleryTitle(artifact: RawArtifactRecord, source?: SourceRecord): string {
  const meta = artifact.metadata_json ?? {};
  if (typeof meta.original_url === "string" && meta.original_url) {
    try {
      const parsed = new URL(meta.original_url);
      return parsed.pathname.split("/").filter(Boolean).pop() ?? parsed.hostname;
    } catch {
      return meta.original_url;
    }
  }
  if (source?.name) return source.name;
  return artifact.id;
}