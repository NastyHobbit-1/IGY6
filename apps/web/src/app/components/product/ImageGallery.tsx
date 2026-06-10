import type { RawArtifactRecord, SourceRecord } from "../types";
import { formatDate, formatBytes } from "../helpers";
import { galleryTitle, isImageArtifact, normalizeContentType } from "./evidence-display";
import { EmptyState } from "../ui/EmptyState";
import { ErrorState } from "../ui/ErrorState";
import { ImageCard } from "./ImageCard";

export type GalleryImageItem = {
  id: string;
  title: string;
  sourceLabel: string;
  contentType: string;
  extension: string;
  status: string;
  statusState: string;
  dateLabel: string;
  sizeLabel: string;
  sizeBytes: number;
  createdAt: string;
  thumbHref: string | null;
  viewHref: string;
};

export function buildGalleryImageItems(
  artifacts: RawArtifactRecord[],
  sourceById: Map<string, SourceRecord>,
): GalleryImageItem[] {
  return artifacts
    .filter(isImageArtifact)
    .map((artifact) => {
      const source = artifact.source_id ? sourceById.get(artifact.source_id) : undefined;
      const meta = artifact.metadata_json ?? {};
      const fileName =
        (typeof meta.filename === "string" ? meta.filename : null)
        ?? (typeof meta.original_url === "string" ? meta.original_url : null);
      const contentType = normalizeContentType({
        mimeType: artifact.mime_type,
        fileName,
        sourceType: source?.source_type,
      });
      const ext = fileName ? fileName.split(".").pop()?.toLowerCase() ?? "" : "";
      const viewHref = `/api/artifacts/${artifact.id}/content`;
      return {
        id: artifact.id,
        title: galleryTitle(artifact, source),
        sourceLabel: source?.name ?? "Local source",
        contentType,
        extension: ext || "image",
        status: "Saved locally",
        statusState: "recorded",
        dateLabel: formatDate(artifact.created_at),
        sizeLabel: formatBytes(artifact.size_bytes),
        sizeBytes: artifact.size_bytes ?? 0,
        createdAt: artifact.created_at,
        thumbHref: viewHref,
        viewHref,
      };
    });
}

export function ImageGallery({
  items,
  error,
}: {
  items: GalleryImageItem[];
  error?: string | null;
}) {
  if (error) {
    return <ErrorState title="Could not load images" message={error} />;
  }

  if (items.length === 0) {
    return <EmptyState label="No images yet. Add information with image files, then check back here." />;
  }

  return (
    <div className="productGallery" data-image-gallery>
      <div className="productGalleryToolbar uiOverflowSafe" data-gallery-toolbar>
        <label className="productField productFieldInline">
          <span>Search</span>
          <input type="search" placeholder="Title, source, or filename" data-gallery-search />
        </label>
        <label className="productField productFieldInline">
          <span>Sort</span>
          <select data-gallery-sort defaultValue="newest">
            <option value="newest">Newest</option>
            <option value="oldest">Oldest</option>
            <option value="source">Source</option>
            <option value="type">File type</option>
            <option value="status">Status</option>
            <option value="size">Size</option>
          </select>
        </label>
        <label className="productField productFieldInline">
          <span>Type</span>
          <select data-gallery-filter-type defaultValue="all">
            <option value="all">All types</option>
            <option value="png">PNG</option>
            <option value="jpg">JPG / JPEG</option>
            <option value="webp">WebP</option>
            <option value="gif">GIF</option>
            <option value="svg">SVG</option>
          </select>
        </label>
        <label className="productField productFieldInline">
          <span>Source</span>
          <select data-gallery-filter-source defaultValue="all">
            <option value="all">All sources</option>
            {Array.from(new Set(items.map((item) => item.sourceLabel))).sort().map((source) => (
              <option key={source} value={source}>
                {source}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="productGalleryGrid" data-gallery-grid>
        {items.map((item) => (
          <ImageCard key={item.id} item={item} />
        ))}
      </div>
      <p className="productHint" data-gallery-count>
        Showing {items.length} image{items.length === 1 ? "" : "s"}
      </p>
    </div>
  );
}