import { proxyToRust } from "@/lib/rust-api";

export async function GET(request: Request): Promise<Response> {
  const url = new URL(request.url);
  const limit = url.searchParams.get("limit") ?? "120";
  return proxyToRust(`/ops/runtime-logs?limit=${encodeURIComponent(limit)}`, {
    method: "GET",
  });
}
