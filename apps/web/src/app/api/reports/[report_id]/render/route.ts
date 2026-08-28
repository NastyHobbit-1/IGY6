import { proxyJsonPost } from "@/lib/rust-api";

export async function POST(
  request: Request,
  context: { params: Promise<{ report_id: string }> },
): Promise<Response> {
  const { report_id } = await context.params;
  return proxyJsonPost(`/reports/${encodeURIComponent(report_id)}/render`, request);
}
