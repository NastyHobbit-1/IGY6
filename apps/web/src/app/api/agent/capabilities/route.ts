const apiBaseUrl = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

export async function GET(): Promise<Response> {
  try {
    const response = await fetch(`${apiBaseUrl}/agent/capabilities`, {
      cache: "no-store",
    });
    const payload = await response.json().catch(() => ({ detail: "Rust API returned a non-JSON response" }));
    return Response.json(payload, { status: response.status });
  } catch (error) {
    return Response.json(
      {
        detail: error instanceof Error ? error.message : "Failed to reach Rust API agent capabilities",
      },
      { status: 502 },
    );
  }
}
