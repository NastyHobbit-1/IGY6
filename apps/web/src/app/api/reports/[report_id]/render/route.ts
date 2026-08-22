import { proxyToRust } from "@/lib/rust-api";

type Params = { params: { report_id: string } };

export async function POST(request: Request, { params }: Params): Promise<Response> {
  const reportId = params.report_id;
  if (!reportId) {
    return Response.json({ detail: "report_id is required" }, { status: 400 });
  }
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }
  return proxyToRust(`/reports/${encodeURIComponent(reportId)}/render`, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

