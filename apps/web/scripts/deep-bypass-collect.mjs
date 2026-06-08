#!/usr/bin/env node
/**
 * Deep bypass collector: Playwright + local browser profile/cookie harvest.
 * Writes ops/deep-bypass-result.json under IGY6_DATA_ROOT for the gateway to consume.
 * No credentials are logged to stdout (only a short status line).
 */
import { chromium } from "playwright";
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(scriptDir, "..", "..", "..");

const requestedUrl = process.env.DEEP_BYPASS_URL?.trim() || process.argv[2]?.trim() || "";
const dataRoot = process.env.IGY6_DATA_ROOT?.trim() || join(repoRoot, "storage");
const outDir = join(dataRoot, "ops");
const outFile = join(outDir, "deep-bypass-result.json");

const LOGIN_WALL_RE =
  /(sign in|log in|login required|subscribe to read|subscription required|create an account|paywall|members only|register to continue|access denied|please log in|sign up to continue|you must be logged in|unlock this article)/i;

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

function browserProfiles() {
  const localAppData = process.env.LOCALAPPDATA || join(homedir(), "AppData", "Local");
  return [
    {
      name: "chrome_profile",
      channel: "chrome",
      userDataDir: join(localAppData, "Google", "Chrome", "User Data"),
    },
    {
      name: "edge_profile",
      channel: "msedge",
      userDataDir: join(localAppData, "Microsoft", "Edge", "User Data"),
    },
  ];
}

function harvestCookiesViaPowerShell(url) {
  if (process.platform !== "win32") return null;
  const psScript = join(repoRoot, "scripts", "harvest-browser-cookies.ps1");
  if (!existsSync(psScript)) return null;
  const run = spawnSync(
    "powershell",
    ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", psScript, "-Url", url],
    { encoding: "utf8", timeout: 60_000 },
  );
  if (run.status !== 0 || !run.stdout?.trim()) return null;
  try {
    return JSON.parse(run.stdout.trim());
  } catch {
    return null;
  }
}

async function tryPlaywrightProfile(profile, seedCookie, seedAuthorization) {
  if (!existsSync(profile.userDataDir)) {
    return null;
  }

  let context;
  try {
    context = await chromium.launchPersistentContext(profile.userDataDir, {
      channel: profile.channel,
      headless: true,
      args: [
        "--profile-directory=Default",
        "--disable-blink-features=AutomationControlled",
        "--no-first-run",
        "--no-default-browser-check",
      ],
      viewport: { width: 1365, height: 900 },
      extraHTTPHeaders: {
        "Accept-Language": "en-US,en;q=0.9",
      },
    });
  } catch {
    return null;
  }

  try {
    const page = await context.newPage();
    if (seedCookie) {
      const domain = urlDomain(requestedUrl);
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

    await page.goto(requestedUrl, {
      waitUntil: "domcontentloaded",
      timeout: 60_000,
      referer: "https://www.google.com/",
    });
    await page.waitForTimeout(2500);

    let html = await page.content();
    let finalUrl = page.url();
    let wallScore = loginWallScore(html);

    if (wallScore > 0) {
      await page.goto(requestedUrl, {
        waitUntil: "networkidle",
        timeout: 60_000,
        referer: "https://t.co/",
      });
      await page.waitForTimeout(2000);
      html = await page.content();
      finalUrl = page.url();
      wallScore = loginWallScore(html);
    }

    const cookies = await context.cookies(finalUrl);
    const cookie = cookieHeaderFromPlaywright(cookies);
    const storageJson = await page.evaluate(() => JSON.stringify(localStorage));
    const authorization =
      seedAuthorization || findBearerInStorage(storageJson) || null;

    return {
      strategy: profile.name,
      final_url: finalUrl,
      cookie: cookie || null,
      authorization,
      html,
      login_wall_score: wallScore,
    };
  } finally {
    await context.close().catch(() => {});
  }
}

async function tryEphemeralPlaywright(seedCookie, seedAuthorization) {
  const browser = await chromium.launch({
    headless: true,
    args: ["--disable-blink-features=AutomationControlled"],
  });
  try {
    const context = await browser.newContext({
      userAgent:
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36",
      extraHTTPHeaders: {
        Referer: "https://www.google.com/",
        "Accept-Language": "en-US,en;q=0.9",
      },
    });
    const page = await context.newPage();
    if (seedCookie) {
      const domain = urlDomain(requestedUrl);
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
    await page.goto(requestedUrl, { waitUntil: "domcontentloaded", timeout: 60_000 });
    await page.waitForTimeout(2000);
    const html = await page.content();
    const cookies = await context.cookies(page.url());
    return {
      strategy: "ephemeral_playwright",
      final_url: page.url(),
      cookie: cookieHeaderFromPlaywright(cookies) || seedCookie || null,
      authorization: seedAuthorization,
      html,
      login_wall_score: loginWallScore(html),
    };
  } finally {
    await browser.close().catch(() => {});
  }
}

async function main() {
  const result = {
    requested_url: requestedUrl,
    final_url: requestedUrl,
    strategy: null,
    cookie: null,
    authorization: null,
    html: null,
    login_wall_score: null,
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

  let best = null;
  let bestScore = -1;

  const consider = (candidate) => {
    if (!candidate?.html || candidate.html.length < 200) return;
    const score = candidate.html.length - candidate.login_wall_score * 8000;
    if (score > bestScore) {
      bestScore = score;
      best = candidate;
    }
  };

  for (const profile of browserProfiles()) {
    try {
      const candidate = await tryPlaywrightProfile(
        profile,
        result.cookie,
        result.authorization,
      );
      if (candidate) consider(candidate);
    } catch (error) {
      result.errors.push(`${profile.name}: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  try {
    const candidate = await tryEphemeralPlaywright(result.cookie, result.authorization);
    if (candidate) consider(candidate);
  } catch (error) {
    result.errors.push(
      `ephemeral_playwright: ${error instanceof Error ? error.message : String(error)}`,
    );
  }

  if (best) {
    result.strategy = best.strategy;
    result.final_url = best.final_url;
    result.cookie = best.cookie || result.cookie;
    result.authorization = best.authorization || result.authorization;
    result.html = best.html;
    result.login_wall_score = best.login_wall_score;
  }

  mkdirSync(outDir, { recursive: true });
  writeFileSync(outFile, JSON.stringify(result, null, 2));
  console.log(
    JSON.stringify({
      ok: true,
      strategy: result.strategy,
      has_cookie: Boolean(result.cookie),
      has_authorization: Boolean(result.authorization),
      has_html: Boolean(result.html),
      login_wall_score: result.login_wall_score,
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
    login_wall_score: null,
    errors: [error instanceof Error ? error.message : String(error)],
  };
  try {
    mkdirSync(outDir, { recursive: true });
    writeFileSync(outFile, JSON.stringify(fallback, null, 2));
  } catch {
    /* ignore */
  }
  console.log(JSON.stringify({ ok: false, reason: "deep_bypass_failed" }));
  process.exit(1);
});