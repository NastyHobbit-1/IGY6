import { proxyJsonPost } from "@/lib/rust-api";

export async function POST(
  request: Request,
  context: { params: Promise<{ experiment_id: string }> },
): Promise<Response> {
  const { experiment_id } = await context.params;
  return proxyJsonPost(`/experiments/${encodeURIComponent(experiment_id)}/status`, request);
}
