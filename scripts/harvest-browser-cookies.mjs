#!/usr/bin/env node
/**
 * Harvest cookies/tokens from local Chrome/Edge profiles (devtools-equivalent)
 * without navigating away from the user's existing sessions.
 */
import { chromium } from "playwright";
import { existsSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const url = process.argv[2]?.trim() || "";
if (!url.startsWith("http://") && !url.startsWith("https://")) {
  process.stdout.write(JSON.stringify({ cookie: null, authorization: null }));
  process.exit(0);
}

const target = new URL(url);
const host = target.hostname.replace(/^www\./, "");
const localAppData = process.env.LOCALAPPDATA || join(homedir(), "AppData", "Local");

const profiles = [
  { channel: "chrome", userDataDir: join(localAppData, "Google", "Chrome", "User Data") },
  { channel: "msedge", userDataDir: join(localAppData, "Microsoft", "Edge", "User Data") },
];

function cookieHeader(cookies) {
  const relevant = cookies.filter(
    (c) =>
      host === c.domain.replace(/^\./, "") ||
      host.endsWith(c.domain.replace(/^\./, "")) ||
      c.domain.replace(/^\./, "").endsWith(host),
  );
  if (!relevant.length) return null;
  return relevant.map((c) => `${c.name}=${c.value}`).join("; ");
}

async function harvestFromProfile(profile) {
  if (!existsSync(profile.userDataDir)) return null;
  let context;
  try {
    context = await chromium.launchPersistentContext(profile.userDataDir, {
      channel: profile.channel,
      headless: true,
      args: ["--profile-directory=Default", "--disable-blink-features=AutomationControlled"],
    });
    const cookies = await context.cookies(url);
    const cookie = cookieHeader(cookies);
    await context.close();
    return cookie ? { cookie, authorization: null } : null;
  } catch {
    await context?.close().catch(() => {});
    return null;
  }
}

let best = { cookie: null, authorization: null };
for (const profile of profiles) {
  const harvested = await harvestFromProfile(profile);
  if (harvested?.cookie && harvested.cookie.length > (best.cookie?.length || 0)) {
    best = harvested;
  }
}

process.stdout.write(JSON.stringify(best));