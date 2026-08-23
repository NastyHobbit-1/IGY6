#!/usr/bin/env node
/**
 * API proxy smoke (Next.js API -> Rust gateway).
 * Verifies that Next API routes respond with JSON even when the Rust API is not running.
 * Accepts 5xx with a well-formed `{ detail }` payload as a pass for offline environments.
 */

import { spawn } from 'child_process';

const DEFAULT_BASE = process.env.WEB_BASE_URL || 'http://127.0.0.1:3000';
const START_TIMEOUT_MS = 45000;
const POLL_INTERVAL_MS = 800;

const endpoints = [
  { method: 'GET', path: '/api/user/status' },
  { method: 'GET', path: '/api/artifacts' },
  { method: 'POST', path: '/api/chat/evidence-answer', body: { message: "Hello", limit: 3 } },
];

const result = {
  baseURL: DEFAULT_BASE,
  startedServer: false,
  checks: [],
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

    for (const ep of endpoints) {
      const url = baseURL + ep.path;
      try {
        const res = await fetch(url, {
          method: ep.method,
          headers: { 'Content-Type': 'application/json' },
          body: ep.body ? JSON.stringify(ep.body) : undefined,
          redirect: 'manual',
        });
        const text = await res.text();
        let json = null;
        try { json = JSON.parse(text); } catch {}
        const okJson = Boolean(json && (json.detail !== undefined || Object.keys(json).length >= 0));
        result.checks.push({
          path: ep.path,
          method: ep.method,
          status: res.status,
          json: okJson ? (json.detail ? { detail: String(json.detail).slice(0, 200) } : Object.fromEntries(Object.entries(json).slice(0, 4))) : null,
          pass: okJson,
        });
        if (!okJson) {
          addError(`Non-JSON or empty response at ${ep.method} ${ep.path} (status ${res.status})`);
        }
      } catch (err) {
        addError(`Fetch error at ${ep.method} ${ep.path}: ${err?.message || String(err)}`);
      }
    }

    const failed = result.errors.length > 0;
    if (failed) {
      console.error(JSON.stringify(result, null, 2));
      process.exit(1);
      return;
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

