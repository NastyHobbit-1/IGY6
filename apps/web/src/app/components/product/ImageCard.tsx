import type { GalleryImageItem } from "./ImageGallery";
import { StatusBadge } from "../ui/StatusBadge";

export function ImageCard({ item }: { item: GalleryImageItem }) {
  return (
    <article
      className="productGalleryCard uiOverflowSafe"
      data-gallery-item
      data-title={item.title.toLowerCase()}
      data-source={item.sourceLabel.toLowerCase()}
      data-extension={item.extension}
      data-status={item.status.toLowerCase()}
      data-timestamp={item.createdAt}
      data-size-bytes={String(item.sizeBytes)}
    >
      <a className="productGalleryThumb" href={item.viewHref} target="_blank" rel="noreferrer">
        {item.thumbHref ? (
          <img src={item.thumbHref} alt={item.title} loading="lazy" data-gallery-thumb />
        ) : null}
        <div className="productGalleryPlaceholder" hidden={Boolean(item.thumbHref)} data-gallery-placeholder>
          <span className="productTypeBadge">{item.contentType}</span>
          <span className="uiClampTitle">{item.title}</span>
        </div>
      </a>
      <div className="productGalleryMeta">
        <span className="productTypeBadge">{item.contentType}</span>
        <h3 className="uiClampTitle" title={item.title}>
          {item.title}
        </h3>
        <p className="productMuted uiBreakLong">{item.sourceLabel}</p>
        <div className="productGalleryMetaRow">
          <StatusBadge state={item.statusState} />
          <span className="productMuted">{item.dateLabel}</span>
        </div>
        <a className="productGhostButton" href={item.viewHref} target="_blank" rel="noreferrer">
          Open image
        </a>
      </div>
    </article>
  );
}