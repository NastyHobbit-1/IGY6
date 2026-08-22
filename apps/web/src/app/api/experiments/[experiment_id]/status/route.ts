import { proxyToRust } from "@/lib/rust-api";

export async function POST(request: Request, { params }: any): Promise<Response> {
  const experimentId = params.experiment_id;
  if (!experimentId) {
    return Response.json({ detail: "experiment_id is required" }, { status: 400 });
  }
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }
  return proxyToRust(`/experiments/${encodeURIComponent(experimentId)}/status`, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

