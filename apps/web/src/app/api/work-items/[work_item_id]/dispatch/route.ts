import { proxyToRust } from "@/lib/rust-api";

type Params = { params: { work_item_id: string } };

export async function POST(_request: Request, { params }: Params): Promise<Response> {
  const workItemId = params.work_item_id;
  if (!workItemId) {
    return Response.json({ detail: "work_item_id is required" }, { status: 400 });
  }
  return proxyToRust(`/work-items/${encodeURIComponent(workItemId)}/dispatch`, {
    method: "POST",
    body: JSON.stringify({}),
  });
}

