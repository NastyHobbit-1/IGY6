import type { ApiResult } from "./types";

export async function postJson<T>(path: string, body: unknown = {}): Promise<ApiResult<T>> {
  const baseUrl = process.env.API_BASE_URL ?? "http://api:8000";

  try {
    const response = await fetch(`${baseUrl}${path}`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
      cache: "no-store",
    });
    if (!response.ok) {
      return { data: null as T, error: `${response.status} ${response.statusText}` };
    }
    return { data: (await response.json()) as T, error: null };
  } catch (error) {
    return {
      data: null as T,
      error: error instanceof Error ? error.message : "Unknown error",
    };
  }
}

export async function getJson<T>(path: string, fallback: T): Promise<ApiResult<T>> {
  const baseUrl = process.env.API_BASE_URL ?? "http://api:8000";

  try {
    const response = await fetch(`${baseUrl}${path}`, {
      cache: "no-store"
    });
    if (!response.ok) {
      return { data: fallback, error: `${response.status} ${response.statusText}` };
    }
    return { data: (await response.json()) as T, error: null };
  } catch (error) {
    return {
      data: fallback,
      error: error instanceof Error ? error.message : "Unknown error"
    };
  }
}
