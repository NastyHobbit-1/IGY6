import { proxyToRust } from "@/lib/rust-api";

export async function POST(): Promise<Response> {
  return proxyToRust("/memory/vector/chunks/ensure", {
    method: "POST",
    body: JSON.stringify({}),
  });
}

