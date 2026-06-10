const apiBaseUrl = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

type RouteContext = {
  params: Promise<{ source_id: string }>;
};

export async function POST(request: Request, context: RouteContext): Promise<Response> {
  let body: unknown;

  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }

  const { source_id: sourceId } = await context.params;

  try {
    const response = await fetch(`${apiBaseUrl}/sources/${encodeURIComponent(sourceId)}/review-state`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    });
    const payload = await response.json().catch(() => ({ detail: "Rust API returned a non-JSON response" }));
    return Response.json(payload, { status: response.status });
  } catch (error) {
    return Response.json(
      {
        detail: error instanceof Error ? error.message : "Failed to reach Rust API source review-state route",
      },
      { status: 502 },
    );
  }
}
