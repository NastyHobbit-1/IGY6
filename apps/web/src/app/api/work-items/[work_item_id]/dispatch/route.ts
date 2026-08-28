import { proxyJsonPost } from "@/lib/rust-api";

export async function POST(
  request: Request,
  context: { params: Promise<{ work_item_id: string }> },
): Promise<Response> {
  const { work_item_id } = await context.params;
  return proxyJsonPost(`/work-items/${encodeURIComponent(work_item_id)}/dispatch`, request);
}
