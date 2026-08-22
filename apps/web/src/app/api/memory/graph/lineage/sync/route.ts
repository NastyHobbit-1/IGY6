import { proxyToRust } from "@/lib/rust-api";

export async function POST(): Promise<Response> {
  return proxyToRust("/memory/graph/lineage/sync", {
    method: "POST",
    body: JSON.stringify({}),
  });
}

