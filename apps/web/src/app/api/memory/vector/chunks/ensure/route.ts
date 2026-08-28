import { proxyJsonPost } from "@/lib/rust-api";

export async function POST(request: Request): Promise<Response> {
  return proxyJsonPost("/memory/vector/chunks/ensure", request);
}
