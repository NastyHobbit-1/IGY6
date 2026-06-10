import type { GalleryImageItem } from "../ImageGallery";
import { ImageGallery } from "../ImageGallery";
import { SectionHeader } from "../../shell/SectionHeader";

export function GallerySection({
  items,
  error,
}: {
  items: GalleryImageItem[];
  error?: string | null;
}) {
  return (
    <section className="productSection" data-product-section="gallery" data-tab-panel="results" hidden id="image-gallery">
      <SectionHeader
        eyebrow="Gallery"
        title="Saved images"
        lead="Browse image evidence separately from other saved information. Search, sort, and filter in one place."
        statusState={items.length > 0 ? "ready" : "empty"}
      />
      <ImageGallery items={items} error={error} />
    </section>
  );
}