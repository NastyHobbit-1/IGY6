import { proxyToRust } from "@/lib/rust-api";

export async function GET(): Promise<Response> {
  return proxyToRust("/bypass-intel/playbook");
}

