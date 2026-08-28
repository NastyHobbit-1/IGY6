import { proxyJsonPost } from "@/lib/rust-api";

export async function POST(
  request: Request,
  context: { params: Promise<{ approval_id: string }> },
): Promise<Response> {
  const { approval_id } = await context.params;
  return proxyJsonPost(`/approvals/${encodeURIComponent(approval_id)}/decision`, request);
}
