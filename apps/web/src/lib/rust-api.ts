export const apiBaseUrl = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

export async function proxyToRust(
  path: string,
  init?: RequestInit,
): Promise<Response> {
  try {
    const response = await fetch(`${apiBaseUrl}${path}`, {
      ...init,
      cache: "no-store",
      headers: {
        "Content-Type": "application/json",
        ...(init?.headers ?? {}),
      },
    });
    const contentType = response.headers.get("content-type") ?? "";
    if (contentType.includes("application/json")) {
      const payload = await response.json().catch(() => ({
        detail: "Rust API returned a non-JSON response",
      }));
      return Response.json(payload, { status: response.status });
    }
    const text = await response.text();
    return new Response(text, { status: response.status });
  } catch (error) {
    return Response.json(
      {
        detail: error instanceof Error ? error.message : `Failed to reach Rust API at ${path}`,
      },
      { status: 502 },
    );
  }
}