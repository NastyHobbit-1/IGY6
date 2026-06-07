import { proxyToRust } from "@/lib/rust-api";

export async function GET(
  _request: Request,
  context: { params: Promise<{ artifact_id: string }> },
): Promise<Response> {
  const { artifact_id } = await context.params;
  return proxyToRust(`/artifacts/${encodeURIComponent(artifact_id)}/content`, {
    method: "GET",
  });
}