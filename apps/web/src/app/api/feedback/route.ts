import { proxyToRust } from "@/lib/rust-api";

export async function POST(request: Request): Promise<Response> {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    return Response.json({ detail: "Invalid JSON body" }, { status: 400 });
  }
  return proxyToRust("/feedback", {
    method: "POST",
    body: JSON.stringify(body),
  });
}

