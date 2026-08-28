import { proxyJsonPost } from "@/lib/rust-api";

export async function POST(
  request: Request,
  context: { params: Promise<{ pattern_id: string }> },
): Promise<Response> {
  const { pattern_id } = await context.params;
  return proxyJsonPost(`/analysis/patterns/${encodeURIComponent(pattern_id)}/review`, request);
}
