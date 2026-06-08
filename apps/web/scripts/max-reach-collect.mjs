#!/usr/bin/env node
/**
 * Max reach collector — strongest built-in tier.
 * Extends deep bypass with CDP attach, headed profiles, scroll/expand, mobile/desktop,
 * optional user proxy, and multi-profile Playwright passes.
 * Writes ops/max-reach-result.json under IGY6_DATA_ROOT.
 */
import { chromium, devices } from "playwright";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import {
  enrichPatreonCandidate,
  isPaidPlatformUrl,
  isPatreonUrl,
  mergePaidPlatformResult,
  patreonContentScore,
} from "./lib/paid-platform-collect.mjs";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(scriptDir, "..", "..", "..");

const requestedUrl = process.env.MAX_REACH_URL?.trim() || process.argv[2]?.trim() || "";
const maxDepth = Number(process.env.MAX_REACH_DEPTH || "1") || 1;
const dataRoot = process.env.IGY6_DATA_ROOT?.trim() || join(repoRoot, "storage");
const outDir = join(dataRoot, "ops");
const outFile = join(outDir, "max-reach-result.json");
const patreonSessionFile = join(outDir, "patreon-session.json");

const LOGIN_WALL_RE =
  /(sign in|log in|login required|subscribe to read|subscription required|create an account|paywall|members only|register to continue|access denied|please log in|sign up to continue|you must be logged in|unlock this article|captcha|verify you are human)/i;

const COOKIE_SELECTORS = [
  "button:has-text('Accept')",
  "button:has-text('Accept all')",
  "button:has-text('I agree')",
  "button:has-text('Got it')",
  "button:has-text('OK')",
  "[aria-label*='accept' i]",
  "#onetrust-accept-btn-handler",
];

const EXPAND_SELECTORS = [
  "button:has-text('Read more')",
  "button:has-text('Show more')",
  "button:has-text('Expand')",
  "a:has-text('Read more')",
  "[data-testid*='expand' i]",
];

function loginWallScore(html) {
  if (!html) return 99;
  const matches = html.match(new RegExp(LOGIN_WALL_RE.source, "gi"));
  return matches ? matches.length : 0;
}

function urlDomain(url) {
  try {
    return new URL(url).hostname.replace(/^www\./, "");
  } catch {
    return "";
  }
}

function cookieHeaderFromPlaywright(cookies) {
  return cookies.map((c) => `${c.name}=${c.value}`).join("; ");
}

function findBearerInStorage(storageJson) {
  try {
    const storage = JSON.parse(storageJson);
    for (const [key, value] of Object.entries(storage)) {
      const lower = key.toLowerCase();
      if (!/(token|auth|jwt|bearer|session)/i.test(lower)) continue;
      const text = String(value || "").trim();
      if (text.length < 20) continue;
      if (/^eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+/i.test(text)) return text;
      if (/^bearer\s+/i.test(text)) return text;
    }
  } catch {
    /* ignore */
  }
  return null;
}

function harvestCookiesViaPowerShell(url) {
  if (process.platform !== "win32") return null;
  const psScript = join(repoRoot, "scripts", "harvest-browser-cookies.ps1");
  if (!existsSync(psScript)) return null;
  const run = spawnSync(
    "powershell",
    ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", psScript, "-Url", url],
    { encoding: "utf8", timeout: 90_000 },
  );
  if (run.status !== 0 || !run.stdout?.trim()) return null;
  try {
    return JSON.parse(run.stdout.trim());
  } catch {
    return null;
  }
}

function browserProfiles() {
  const localAppData = process.env.LOCALAPPDATA || join(homedir(), "AppData", "Local");
  const headed = process.env.MAX_REACH_HEADED === "1" || process.env.MAX_REACH_HEADED === "true";
  const profiles = ["Default", "Profile 1", "Profile 2"];
  const channels = [
    { name: "chrome", channel: "chrome", userDataDir: join(localAppData, "Google", "Chrome", "User Data") },
    { name: "edge", channel: "msedge", userDataDir: join(localAppData, "Microsoft", "Edge", "User Data") },
  ];
  const out = [];
  for (const base of channels) {
    for (const profileDir of profiles) {
      out.push({
        name: `${base.name}_${profileDir.replace(/\s+/g, "_").toLowerCase()}`,
        channel: base.channel,
        userDataDir: base.userDataDir,
        profileDirectory: profileDir,
        headed,
      });
    }
  }
  return out;
}

function playwrightProxy() {
  const raw = process.env.MAX_REACH_PROXY?.trim() || process.env.HTTP_PROXY?.trim() || "";
  if (!raw) return undefined;
  try {
    const parsed = new URL(raw);
    return { server: `${parsed.protocol}//${parsed.host}` };
  } catch {
    return { server: raw };
  }
}

async function dismissOverlays(page) {
  for (const selector of COOKIE_SELECTORS) {
    try {
      const node = page.locator(selector).first();
      if (await node.isVisible({ timeout: 800 })) {
        await node.click({ timeout: 2000 });
        await page.waitForTimeout(500);
      }
    } catch {
      /* ignore */
    }
  }
}

async function expandContent(page) {
  for (const selector of EXPAND_SELECTORS) {
    try {
      const node = page.locator(selector).first();
      if (await node.isVisible({ timeout: 600 })) {
        await node.click({ timeout: 1500 });
        await page.waitForTimeout(800);
      }
    } catch {
      /* ignore */
    }
  }
}

async function scrollPage(page) {
  try {
    await page.evaluate(async () => {
      await new Promise((resolve) => {
        let total = 0;
        const step = 400;
        const timer = setInterval(() => {
          window.scrollBy(0, step);
          total += step;
          if (total >= document.body.scrollHeight || total > 12000) {
            clearInterval(timer);
            resolve(null);
          }
        }, 120);
      });
    });
    await page.waitForTimeout(1200);
  } catch {
    /* ignore */
  }
}

async function extractPagePayload(page) {
  const html = await page.content();
  const visibleText = await page
    .evaluate(() => document.body?.innerText?.slice(0, 500_000) || "")
    .catch(() => "");
  const finalUrl = page.url();
  const cookies = await contextSafeCookies(page);
  const storageJson = await page.evaluate(() => JSON.stringify(localStorage)).catch(() => "{}");
  const authorization = findBearerInStorage(storageJson);
  return {
    html,
    visible_text: visibleText,
    final_url: finalUrl,
    cookie: cookieHeaderFromPlaywright(cookies),
    authorization,
    login_wall_score: loginWallScore(html + "\n" + visibleText),
  };
}

async function contextSafeCookies(page) {
  try {
    return await page.context().cookies(page.url());
  } catch {
    return [];
  }
}

async function seedCookies(context, seedCookie, url) {
  if (!seedCookie) return;
  const domain = urlDomain(url);
  const pairs = seedCookie.split(";").map((p) => p.trim()).filter(Boolean);
  const cookies = pairs
    .map((pair) => {
      const idx = pair.indexOf("=");
      if (idx <= 0) return null;
      return {
        name: pair.slice(0, idx).trim(),
        value: pair.slice(idx + 1).trim(),
        domain,
        path: "/",
      };
    })
    .filter(Boolean);
  if (cookies.length) await context.addCookies(cookies);
}

async function navigateAndHarvest(page, url, referer) {
  await page.goto(url, {
    waitUntil: "domcontentloaded",
    timeout: 90_000,
    referer: referer || "https://www.google.com/",
  });
  await page.waitForTimeout(2000);
  await dismissOverlays(page);
  await expandContent(page);
  await scrollPage(page);
  try {
    await page.waitForLoadState("networkidle", { timeout: 12_000 });
  } catch {
    /* ignore */
  }
  let payload = await extractPagePayload(page);
  if (payload.login_wall_score > 0) {
    await page.goto(url, {
      waitUntil: "networkidle",
      timeout: 90_000,
      referer: "https://t.co/",
    });
    await page.waitForTimeout(2500);
    await dismissOverlays(page);
    await expandContent(page);
    await scrollPage(page);
    payload = await extractPagePayload(page);
  }
  return payload;
}

async function tryPlaywrightProfile(profile, seedCookie, seedAuthorization, viewportLabel, viewport) {
  if (!existsSync(profile.userDataDir)) return null;
  const proxy = playwrightProxy();
  const launchOpts = {
    channel: profile.channel,
    headless: !profile.headed,
    args: [
      `--profile-directory=${profile.profileDirectory}`,
      "--disable-blink-features=AutomationControlled",
      "--no-first-run",
      "--no-default-browser-check",
    ],
    viewport,
    extraHTTPHeaders: { "Accept-Language": "en-US,en;q=0.9" },
    proxy,
  };

  let context;
  try {
    context = await chromium.launchPersistentContext(profile.userDataDir, launchOpts);
  } catch {
    return null;
  }

  try {
    const page = await context.newPage();
    await seedCookies(context, seedCookie, requestedUrl);
    const payload = await navigateAndHarvest(page, requestedUrl, "https://www.google.com/");
    const enriched = await enrichPatreonCandidate(page, payload, requestedUrl);
    return {
      strategy: `${profile.name}_${viewportLabel}`,
      ...enriched,
      authorization: enriched.authorization || seedAuthorization || null,
      cookie: enriched.cookie || seedCookie || null,
    };
  } finally {
    await context.close().catch(() => {});
  }
}

async function tryEphemeralPlaywright(seedCookie, seedAuthorization, viewportLabel, viewport, mobile) {
  const proxy = playwrightProxy();
  const browser = await chromium.launch({
    headless: process.env.MAX_REACH_HEADED !== "1" && process.env.MAX_REACH_HEADED !== "true",
    args: ["--disable-blink-features=AutomationControlled"],
    proxy,
  });
  try {
    const context = await browser.newContext({
      ...devices[mobile ? "Pixel 7" : "Desktop Chrome"],
      viewport,
      userAgent: mobile
        ? devices["Pixel 7"].userAgent
        : "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
      extraHTTPHeaders: {
        Referer: "https://www.google.com/",
        "Accept-Language": "en-US,en;q=0.9",
      },
    });
    const page = await context.newPage();
    await seedCookies(context, seedCookie, requestedUrl);
    const payload = await navigateAndHarvest(page, requestedUrl, "https://www.google.com/");
    const enriched = await enrichPatreonCandidate(page, payload, requestedUrl);
    return {
      strategy: `ephemeral_${viewportLabel}${mobile ? "_mobile" : "_desktop"}`,
      ...enriched,
      authorization: enriched.authorization || seedAuthorization || null,
      cookie: enriched.cookie || seedCookie || null,
    };
  } finally {
    await browser.close().catch(() => {});
  }
}

async function tryCdpAttach(seedCookie, seedAuthorization) {
  const port = process.env.MAX_REACH_CDP_PORT?.trim() || "9222";
  const endpoint = `http://127.0.0.1:${port}`;
  let browser;
  try {
    browser = await chromium.connectOverCDP(endpoint);
  } catch {
    return null;
  }
  try {
    const context = browser.contexts()[0] || (await browser.newContext());
    const page = context.pages()[0] || (await context.newPage());
    await seedCookies(context, seedCookie, requestedUrl);
    const payload = await navigateAndHarvest(page, requestedUrl, "https://www.google.com/");
    const enriched = await enrichPatreonCandidate(page, payload, requestedUrl);
    return {
      strategy: "cdp_attached_chrome",
      ...enriched,
      authorization: enriched.authorization || seedAuthorization || null,
      cookie: enriched.cookie || seedCookie || null,
    };
  } catch {
    return null;
  }
}

async function discoverSameHostLinks(pageUrl, html) {
  const links = new Set();
  try {
    const origin = new URL(pageUrl).origin;
    const hrefRe = /href=["']([^"'#]+)["']/gi;
    let match;
    while ((match = hrefRe.exec(html)) !== null) {
      try {
        const resolved = new URL(match[1], pageUrl);
        if (resolved.origin === origin && resolved.href.startsWith("http")) {
          links.add(resolved.href);
        }
      } catch {
        /* ignore */
      }
    }
  } catch {
    /* ignore */
  }
  return [...links].slice(0, 40);
}

function loadPatreonSession() {
  try {
    if (!existsSync(patreonSessionFile)) return null;
    return JSON.parse(readFileSync(patreonSessionFile, "utf8"));
  } catch {
    return null;
  }
}

function savePatreonSession(cookie, authorization, strategy) {
  if (!cookie && !authorization) return;
  try {
    mkdirSync(outDir, { recursive: true });
    writeFileSync(
      patreonSessionFile,
      JSON.stringify(
        {
          updated_at: new Date().toISOString(),
          cookie: cookie || null,
          authorization: authorization || null,
          strategy: strategy || null,
        },
        null,
        2,
      ),
    );
  } catch {
    /* ignore */
  }
}

function scoreCandidate(candidate) {
  if (!candidate) return -1;
  if (isPatreonUrl(requestedUrl) || candidate.paid_platform === "patreon") {
    return patreonContentScore(candidate);
  }
  const textLen = (candidate.visible_text || "").length;
  const htmlLen = (candidate.html || "").length;
  const authBonus = candidate.authorization ? 5000 : 0;
  const cookieBonus = candidate.cookie ? 2500 : 0;
  return Math.max(textLen, htmlLen) + authBonus + cookieBonus - candidate.login_wall_score * 10_000;
}

async function main() {
  const result = {
    requested_url: requestedUrl,
    final_url: requestedUrl,
    strategy: null,
    cookie: null,
    authorization: null,
    html: null,
    visible_text: null,
    login_wall_score: null,
    discovered_links: [],
    media_urls: [],
    api_bodies: [],
    post_id: null,
    paid_platform: isPaidPlatformUrl(requestedUrl) ? (isPatreonUrl(requestedUrl) ? "patreon" : "paid_platform") : null,
    passes_attempted: 0,
    max_depth: maxDepth,
    errors: [],
  };

  if (!requestedUrl.startsWith("http://") && !requestedUrl.startsWith("https://")) {
    result.errors.push("invalid_url");
    mkdirSync(outDir, { recursive: true });
    writeFileSync(outFile, JSON.stringify(result, null, 2));
    console.log(JSON.stringify({ ok: false, reason: "invalid_url" }));
    process.exit(1);
  }

  const harvested = harvestCookiesViaPowerShell(requestedUrl);
  if (harvested?.cookie) {
    result.cookie = harvested.cookie;
    result.authorization = harvested.authorization || null;
    result.strategy = "devtools_cookie_harvest";
  }
  if (isPatreonUrl(requestedUrl)) {
    const saved = loadPatreonSession();
    if (saved?.cookie && !result.cookie) result.cookie = saved.cookie;
    if (saved?.authorization && !result.authorization) result.authorization = saved.authorization;
  }

  let best = null;
  let bestScore = -1;
  const consider = (candidate) => {
    result.passes_attempted += 1;
    const score = scoreCandidate(candidate);
    const hasPatreonMedia = Array.isArray(candidate?.media_urls) && candidate.media_urls.length > 0;
    const hasHtml = candidate?.html && candidate.html.length >= 200;
    if (score > bestScore && (hasHtml || hasPatreonMedia)) {
      bestScore = score;
      best = candidate;
    }
  };

  const viewports = [
    ["desktop_hd", { width: 1920, height: 1080 }],
    ["laptop", { width: 1366, height: 900 }],
    ["tablet", { width: 834, height: 1112 }],
  ];

  try {
    const cdp = await tryCdpAttach(result.cookie, result.authorization);
    if (cdp) consider(cdp);
  } catch (error) {
    result.errors.push(`cdp: ${error instanceof Error ? error.message : String(error)}`);
  }

  for (const profile of browserProfiles()) {
    for (const [label, viewport] of viewports) {
      try {
        const candidate = await tryPlaywrightProfile(
          profile,
          result.cookie,
          result.authorization,
          label,
          viewport,
        );
        if (candidate) consider(candidate);
      } catch (error) {
        result.errors.push(
          `${profile.name}/${label}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }

  for (const [label, viewport] of viewports) {
    for (const mobile of [false, true]) {
      try {
        const candidate = await tryEphemeralPlaywright(
          result.cookie,
          result.authorization,
          label,
          viewport,
          mobile,
        );
        if (candidate) consider(candidate);
      } catch (error) {
        result.errors.push(
          `ephemeral/${label}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }

  if (best) {
    mergePaidPlatformResult(result, best);
    result.strategy = best.strategy;
    result.final_url = best.final_url;
    result.cookie = best.cookie || result.cookie;
    result.authorization = best.authorization || result.authorization;
    result.html = best.html;
    result.visible_text = best.visible_text || null;
    result.login_wall_score = best.login_wall_score;
    result.discovered_links = discoverSameHostLinks(result.final_url, result.html || "");
    if (Array.isArray(best.media_urls) && best.media_urls.length) {
      result.discovered_links = [...new Set([...best.media_urls, ...result.discovered_links])];
    }
    if (maxDepth > 0) {
      result.discovered_links = result.discovered_links.slice(0, 10 * maxDepth);
    }
    if (isPatreonUrl(requestedUrl) && (result.cookie || result.authorization)) {
      savePatreonSession(result.cookie, result.authorization, result.strategy);
    }
  }

  mkdirSync(outDir, { recursive: true });
  writeFileSync(outFile, JSON.stringify(result, null, 2));
  console.log(
    JSON.stringify({
      ok: Boolean(best),
      strategy: result.strategy,
      has_cookie: Boolean(result.cookie),
      has_authorization: Boolean(result.authorization),
      has_html: Boolean(result.html),
      has_visible_text: Boolean(result.visible_text),
      login_wall_score: result.login_wall_score,
      passes_attempted: result.passes_attempted,
      discovered_links: result.discovered_links.length,
    }),
  );
}

main().catch((error) => {
  const fallback = {
    requested_url: requestedUrl,
    final_url: requestedUrl,
    strategy: null,
    cookie: null,
    authorization: null,
    html: null,
    visible_text: null,
    login_wall_score: null,
    discovered_links: [],
    passes_attempted: 0,
    max_depth: maxDepth,
    errors: [error instanceof Error ? error.message : String(error)],
  };
  try {
    mkdirSync(outDir, { recursive: true });
    writeFileSync(outFile, JSON.stringify(fallback, null, 2));
  } catch {
    /* ignore */
  }
  console.log(JSON.stringify({ ok: false, reason: "max_reach_failed" }));
  process.exit(1);
});