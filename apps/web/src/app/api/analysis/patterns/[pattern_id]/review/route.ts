import { proxyToRust } from "@/lib/rust-api";

type Params = { params: { pattern_id: string } };

export async function POST(request: Request, { params }: Params): Promise<Response> {
  const patternId = params.pattern_id;
  if (!patternId) {
    return Response.json({ detail: "pattern_id is required" }, { status: 400 });
  }
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }
  return proxyToRust(`/analysis/patterns/${encodeURIComponent(patternId)}/review`, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

