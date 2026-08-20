import { proxyToRust } from "@/lib/rust-api";

export async function POST(request: Request): Promise<Response> {
  let body: unknown = {};
  try {
    body = await request.json();
  } catch {
    body = {};
  }
  return proxyToRust("/ops/runtime-logs/append", {
    method: "POST",
    body: JSON.stringify(body),
  });
}
