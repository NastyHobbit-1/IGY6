/**
 * Paid-platform helpers (Patreon-first): API/media extraction with logged-in browser sessions.
 */

export const PAID_PLATFORM_HOSTS = [
  "patreon.com",
  "patreonusercontent.com",
  "onlyfans.com",
  "fansly.com",
];

export function isPaidPlatformUrl(url) {
  try {
    const host = new URL(url).hostname.replace(/^www\./, "").toLowerCase();
    return PAID_PLATFORM_HOSTS.some((entry) => host === entry || host.endsWith(`.${entry}`));
  } catch {
    return /patreon|onlyfans|fansly/i.test(url);
  }
}

export function isPatreonUrl(url) {
  try {
    const host = new URL(url).hostname.replace(/^www\./, "").toLowerCase();
    return host.includes("patreon");
  } catch {
    return /patreon/i.test(url);
  }
}

export function patreonPostIdFromUrl(url) {
  try {
    const parsed = new URL(url);
    const parts = parsed.pathname.split("/").filter(Boolean);
    const postsIdx = parts.indexOf("posts");
    if (postsIdx >= 0 && parts[postsIdx + 1]) {
      const slug = parts[postsIdx + 1];
      const trailing = slug.match(/(\d{4,})$/);
      if (trailing) return trailing[1];
      if (/^\d+$/.test(slug)) return slug;
    }
  } catch {
    /* ignore */
  }
  const fallback = url.match(/patreon\.com\/posts\/[^/?#]*?(\d{4,})/i);
  return fallback ? fallback[1] : null;
}

export function extractUrlsFromText(text) {
  const urls = new Set();
  const re = /https?:\/\/[^\s"'<>]+/gi;
  let match;
  while ((match = re.exec(text || "")) !== null) {
    const cleaned = match[0].replace(/[),.;]+$/g, "");
    if (cleaned.startsWith("http")) urls.add(cleaned);
  }
  return [...urls];
}

export function extractPatreonMediaUrls(html, visibleText, apiBodies = []) {
  const media = new Set();
  for (const blob of [html, visibleText, ...apiBodies]) {
    for (const url of extractUrlsFromText(blob)) {
      if (/patreonusercontent\.com/i.test(url) || /\.(jpg|jpeg|png|gif|webp|mp4|m4v|mov)(\?|$)/i.test(url)) {
        media.add(url.replace(/&amp;/g, "&"));
      }
    }
    const cdnRe = /https?:\/\/[^"'\\s]*patreonusercontent\.com[^"'\\s]*/gi;
    let cdnMatch;
    while ((cdnMatch = cdnRe.exec(blob || "")) !== null) {
      media.add(cdnMatch[0].replace(/&amp;/g, "&"));
    }
  }
  return [...media];
}

export async function capturePatreonApiResponses(page, timeoutMs = 12_000) {
  const bodies = [];
  const handler = async (response) => {
    try {
      const url = response.url();
      if (!/patreon\.com\/api\//i.test(url)) return;
      if (!response.ok()) return;
      const contentType = response.headers()["content-type"] || "";
      if (!contentType.includes("json")) return;
      const text = await response.text();
      if (text.length > 50) bodies.push(text);
    } catch {
      /* ignore */
    }
  };
  page.on("response", handler);
  await page.waitForTimeout(timeoutMs);
  page.off("response", handler);
  return bodies;
}

export async function fetchPatreonApiWithSession(page, postId, cookieHeader) {
  if (!postId) return [];
  const endpoints = [
    `https://www.patreon.com/api/posts/${postId}?include=attachments,audio,user,images,media&fields[post]=content,title,post_type&json-api-version=1.0`,
    `https://www.patreon.com/api/posts/${postId}?include=attachments,user&json-api-version=1.0`,
  ];
  const results = [];
  for (const endpoint of endpoints) {
    try {
      const payload = await page.evaluate(
        async ({ endpoint, cookieHeader }) => {
          const headers = {
            Accept: "application/json",
            "Accept-Language": "en-US,en;q=0.9",
          };
          if (cookieHeader) headers.Cookie = cookieHeader;
          const response = await fetch(endpoint, {
            credentials: "include",
            headers,
          });
          if (!response.ok) return null;
          return await response.text();
        },
        { endpoint, cookieHeader },
      );
      if (payload && payload.length > 50) results.push(payload);
    } catch {
      /* ignore */
    }
  }
  return results;
}

export function patreonContentScore(candidate) {
  if (!candidate) return -1;
  const mediaCount = Array.isArray(candidate.media_urls) ? candidate.media_urls.length : 0;
  const apiCount = Array.isArray(candidate.api_bodies) ? candidate.api_bodies.length : 0;
  const textLen = (candidate.visible_text || candidate.html || "").length;
  const authBonus = candidate.authorization ? 8000 : 0;
  const cookieBonus = candidate.cookie ? 5000 : 0;
  const mediaBonus = mediaCount * 12_000;
  const apiBonus = apiCount * 4000;
  const wallPenalty = (candidate.login_wall_score || 0) * 12_000;
  return textLen + authBonus + cookieBonus + mediaBonus + apiBonus - wallPenalty;
}

export async function enrichPatreonCandidate(page, candidate, requestedUrl) {
  if (!isPatreonUrl(requestedUrl)) return candidate;
  const postId = patreonPostIdFromUrl(requestedUrl) || patreonPostIdFromUrl(candidate?.final_url || "");
  const apiFromNetwork = await capturePatreonApiResponses(page, 1500);
  const apiFromFetch = await fetchPatreonApiWithSession(page, postId, candidate?.cookie || "");
  const apiBodies = [...apiFromNetwork, ...apiFromFetch];
  const mediaUrls = extractPatreonMediaUrls(candidate?.html || "", candidate?.visible_text || "", apiBodies);
  return {
    ...candidate,
    post_id: postId,
    api_bodies: apiBodies,
    media_urls: mediaUrls,
    paid_platform: "patreon",
  };
}

export function mergePaidPlatformResult(result, candidate) {
  if (!candidate) return result;
  result.paid_platform = candidate.paid_platform || (isPatreonUrl(result.requested_url) ? "patreon" : null);
  result.post_id = candidate.post_id || result.post_id || null;
  result.api_bodies = candidate.api_bodies || result.api_bodies || [];
  result.media_urls = candidate.media_urls || result.media_urls || [];
  return result;
}