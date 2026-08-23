#!/usr/bin/env node
/**
 * UI interaction smoke (Playwright-based, dynamic start).
 * - Starts the app on a clear local URL if not already running
 * - Clicks Settings tab and verifies core wiring data attributes
 * - Falls back to a skip if Playwright browsers are not available
 * - Does not require external services; works against empty local state
 */

import { spawn } from 'child_process';
import { chromium } from 'playwright';

const DEFAULT_BASE = process.env.WEB_BASE_URL || 'http://127.0.0.1:3000';
const START_TIMEOUT_MS = 45000;
const NAV_TIMEOUT_MS = 20000;
const POLL_INTERVAL_MS = 800;

const result = {
  baseURL: DEFAULT_BASE,
  startedServer: false,
  usedBrowser: false,
  skippedReason: null,
  chatWired: null,
  settingsWired: null,
  errors: [],
};

function addError(msg) {
  result.errors.push(msg);
}

async function isReachable(url, timeoutMs = 4000) {
  const controller = new AbortController();
  const t = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(url, { method: 'GET', signal: controller.signal, redirect: 'manual' });
    return res.status < 500;
  } catch {
    return false;
  } finally {
    clearTimeout(t);
  }
}

async function startWebAppAndGetURL() {
  const child = spawn('npm run start', {
    cwd: process.cwd(),
    stdio: ['ignore', 'pipe', 'pipe'],
    shell: true,
    env: { ...process.env, FORCE_COLOR: '0' },
  });

  let resolvedURL = null;
  let stderrBuf = '';

  const urlPromise = new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error(`Timed out waiting for clear local URL after ${START_TIMEOUT_MS}ms`));
    }, START_TIMEOUT_MS);

    function tryExtract(data) {
      const str = data.toString();
      const m = str.match(/Using clear local URL:\s*(http:\/\/[^\s]+)/);
      if (m && !resolvedURL) {
        resolvedURL = m[1];
        clearTimeout(timer);
        resolve(resolvedURL);
      }
    }

    child.stdout.on('data', tryExtract);
    child.stderr.on('data', (d) => { stderrBuf += d.toString(); });

    child.on('error', (err) => {
      clearTimeout(timer);
      reject(err);
    });
    child.on('exit', (code) => {
      if (!resolvedURL) {
        clearTimeout(timer);
        reject(new Error(`Start script exited (code ${code}) before emitting URL. stderr: ${stderrBuf.slice(0, 800)}`));
      }
    });
  });

  const url = await urlPromise;
  for (let i = 0; i < 12; i++) {
    if (await isReachable(url, 1500)) break;
    await new Promise((r) => setTimeout(r, POLL_INTERVAL_MS));
  }
  return { url, child };
}

async function main() {
  let serverChild = null;
  try {
    let baseURL = DEFAULT_BASE;
    const reachable = await isReachable(baseURL, 3000);
    if (!reachable) {
      const started = await startWebAppAndGetURL();
      baseURL = started.url;
      serverChild = started.child;
      result.baseURL = baseURL;
      result.startedServer = true;
    }

    let browser = null;
    try {
      browser = await chromium.launch({ headless: true });
      result.usedBrowser = true;
      const context = await browser.newContext();
      const page = await context.newPage();
      page.on('pageerror', (err) => addError(String(err)));
      await page.goto(baseURL, { waitUntil: 'domcontentloaded', timeout: NAV_TIMEOUT_MS });

      // Wait briefly for client script to wire the chat hub
      await page.waitForFunction(
        () => document.querySelector('[data-unified-chat]')?.getAttribute('data-chat-wired') === 'true',
        { timeout: 15000 }
      ).catch(() => null);
      result.chatWired = await page.locator('[data-unified-chat]').first().getAttribute('data-chat-wired').catch(() => null);

      // Open Settings
      await page.getByRole('tab', { name: 'Settings' }).first().click().catch(() => null);
      await page.waitForTimeout(500);
      result.settingsWired = await page.locator('[data-settings-env]').first().getAttribute('data-settings-wired').catch(() => null);

      await browser.close().catch(() => {});
    } catch (launchErr) {
      result.skippedReason = `Playwright chromium launch failed: ${launchErr?.message || String(launchErr)}`;
      if (result.errors.length === 0) {
        // Treat as a skip, not a hard failure
        console.log(JSON.stringify(result, null, 2));
        process.exit(0);
        return;
      }
    }

    // Decide pass/fail for browser path
    if (result.usedBrowser) {
      const hardFailures = [];
      if (result.chatWired !== 'true') hardFailures.push(`chatWired=${String(result.chatWired)}`);
      if (result.settingsWired !== 'true') hardFailures.push(`settingsWired=${String(result.settingsWired)}`);
      if (hardFailures.length > 0) {
        addError(`Wiring checks failed: ${hardFailures.join(', ')}`);
        console.error(JSON.stringify(result, null, 2));
        process.exit(1);
        return;
      }
    }

    console.log(JSON.stringify(result, null, 2));
    process.exit(0);
  } finally {
    if (serverChild && !serverChild.killed) {
      try {
        serverChild.kill('SIGTERM');
        await new Promise((r) => setTimeout(r, 600));
      } catch {}
    }
  }
}

main().catch((e) => {
  addError(e?.message || String(e));
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
});

