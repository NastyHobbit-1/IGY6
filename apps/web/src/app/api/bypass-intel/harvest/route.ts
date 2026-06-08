import { proxyToRust } from "@/lib/rust-api";

export async function POST(request: Request): Promise<Response> {
  let body: unknown;
  try {
    body = await request.json();
  } catch {
    body = { force: true, requested_by_actor_id: "local-owner" };
  }
  return proxyToRust("/bypass-intel/harvest", {
    method: "POST",
    body: JSON.stringify(body),
  });
}