export const apiBaseUrl = process.env.API_BASE_URL ?? "http://127.0.0.1:8000";

/** Browser-side UI must call Next `/api` proxies, never the Rust origin directly. */
export const browserApiBaseUrl = "/api";

export async function proxyJsonPost(path: string, request: Request): Promise<Response> {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }
  return proxyToRust(path, {
    method: "POST",
    body: JSON.stringify(body),
  });
}

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
