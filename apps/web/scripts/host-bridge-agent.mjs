#!/usr/bin/env node
/**
 * Local ensure agent on 127.0.0.1 — started with host bridge.
 * UI max reach calls this to auto-start bridge + Playwright before collection.
 */
import http from "node:http";
import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, unlinkSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(scriptDir, "..", "..", "..");
const ensureScript = join(repoRoot, "scripts", "ensure-host-bridge.ps1");
const port = Number(process.env.IGY6_HOST_BRIDGE_AGENT_PORT || "8770");
function readEnvValue(key) {
  const envPath = join(repoRoot, ".env");
  if (!existsSync(envPath)) return null;
  for (const line of readFileSync(envPath, "utf8").split(/\r?\n/)) {
    const match = line.match(new RegExp(`^\\s*${key}=(.*)$`));
    if (match) return match[1].trim().replace(/\//g, "\\");
  }
  return null;
}

const dataRoot =
  process.env.IGY6_DATA_ROOT?.trim() ||
  readEnvValue("IGY6_DATA_ROOT") ||
  join(repoRoot, "..", "..", "IGY6_Data");
const opsDir = join(dataRoot, "ops");
const triggerFile = join(opsDir, "max-reach-ensure.requested");
const pidFile = join(opsDir, "host-bridge-agent.pid");

let ensureRunning = false;

function corsHeaders(origin) {
  const allowed =
    !origin ||
    /^https?:\/\/(127\.0\.0\.1|localhost)(:\d+)?$/i.test(origin);
  return {
    "Access-Control-Allow-Origin": allowed ? origin || "*" : "http://127.0.0.1:3002",
    "Access-Control-Allow-Methods": "GET, POST, OPTIONS",
    "Access-Control-Allow-Headers": "Content-Type",
    "Content-Type": "application/json",
  };
}

function runEnsure(maxReach) {
  if (ensureRunning) {
    return Promise.resolve({ ok: true, status: "already_running" });
  }
  ensureRunning = true;
  return new Promise((resolve) => {
    const args = [
      "-NoProfile",
      "-ExecutionPolicy",
      "Bypass",
      "-File",
      ensureScript,
      "-Quiet",
    ];
    if (maxReach) args.push("-MaxReach");
    const child = spawn("powershell", args, {
      cwd: repoRoot,
      env: { ...process.env, IGY6_DATA_ROOT: dataRoot },
      windowsHide: true,
    });
    let stderr = "";
    child.stderr?.on("data", (chunk) => {
      stderr += String(chunk);
    });
    child.on("close", (code) => {
      ensureRunning = false;
      resolve({
        ok: code === 0,
        status: code === 0 ? "ready" : "failed",
        exit_code: code,
        stderr: stderr.slice(0, 500) || null,
      });
    });
  });
}

async function handleEnsure(maxReach) {
  const result = await runEnsure(maxReach);
  return result;
}

function pollTriggerFile() {
  if (!existsSync(triggerFile)) return;
  try {
    unlinkSync(triggerFile);
  } catch {
    return;
  }
  void handleEnsure(true);
}

const server = http.createServer(async (req, res) => {
  const origin = req.headers.origin || "";
  const headers = corsHeaders(origin);

  if (req.method === "OPTIONS") {
    res.writeHead(204, headers);
    res.end();
    return;
  }

  if (req.method === "GET" && req.url === "/health") {
    res.writeHead(200, headers);
    res.end(JSON.stringify({ ok: true, agent: "host-bridge-agent", port }));
    return;
  }

  if (req.method === "POST" && (req.url === "/ensure-max-reach" || req.url === "/ensure")) {
    const maxReach = req.url === "/ensure-max-reach";
    const result = await handleEnsure(maxReach);
    res.writeHead(result.ok ? 200 : 500, headers);
    res.end(JSON.stringify(result));
    return;
  }

  res.writeHead(404, headers);
  res.end(JSON.stringify({ ok: false, detail: "not_found" }));
});

mkdirSync(opsDir, { recursive: true });
writeFileSync(pidFile, String(process.pid));
setInterval(pollTriggerFile, 1500);

const HARVEST_INTERVAL_MS = 6 * 60 * 60 * 1000;
const HARVEST_START_DELAY_MS = 2 * 60 * 1000;

async function triggerBypassIntelHarvest(force = false) {
  const apiPort = process.env.APP_PORT?.trim() || readEnvValue("APP_PORT") || "8002";
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), 300000);
  try {
    const response = await fetch(`http://127.0.0.1:${apiPort}/bypass-intel/harvest`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        requested_by_actor_id: "bypass-intel-scheduler",
        force,
      }),
      signal: controller.signal,
    });
    if (!response.ok) {
      const payload = await response.json().catch(() => ({}));
      console.warn("Bypass intel harvest:", payload?.detail || response.status);
    }
  } catch (error) {
    console.warn("Bypass intel harvest unavailable:", error);
  } finally {
    clearTimeout(timeout);
  }
}

setTimeout(() => void triggerBypassIntelHarvest(false), HARVEST_START_DELAY_MS);
setInterval(() => void triggerBypassIntelHarvest(false), HARVEST_INTERVAL_MS);

server.listen(port, "127.0.0.1", () => {
  console.log(
    JSON.stringify({
      ok: true,
      agent: "host-bridge-agent",
      listen: `http://127.0.0.1:${port}`,
      data_root: dataRoot,
    }),
  );
});

process.on("SIGINT", () => {
  try {
    if (existsSync(pidFile)) unlinkSync(pidFile);
  } catch {
    /* ignore */
  }
  process.exit(0);
});