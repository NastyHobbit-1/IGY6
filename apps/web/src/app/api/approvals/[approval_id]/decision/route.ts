const apiBaseUrl = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

type Params = { params: { approval_id: string } };

export async function POST(request: Request, { params }: Params): Promise<Response> {
  const approvalId = params.approval_id;
  if (!approvalId) {
    return Response.json({ detail: "approval_id is required" }, { status: 400 });
  }
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }
  try {
    const response = await fetch(`${apiBaseUrl}/approvals/${encodeURIComponent(approvalId)}/decision`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    });
    const payload = await response.json().catch(() => ({ detail: "Rust API returned a non-JSON response" }));
    return Response.json(payload, { status: response.status });
  } catch (error) {
    return Response.json(
      { detail: error instanceof Error ? error.message : "Failed to reach Rust API /approvals/:id/decision" },
      { status: 502 },
    );
  }
}

