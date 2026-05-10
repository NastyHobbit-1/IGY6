const apiBaseUrl = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

export async function POST(request: Request): Promise<Response> {
  let body: unknown;

  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }

  try {
    const response = await fetch(`${apiBaseUrl}/settings/env/verify`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    });

    const payload = await response.json().catch(() => ({ detail: "FastAPI returned a non-JSON response" }));
    return Response.json(payload, { status: response.status });
  } catch (error) {
    return Response.json(
      {
        detail: error instanceof Error ? error.message : "Failed to reach FastAPI settings verify endpoint",
      },
      { status: 502 },
    );
  }
}
