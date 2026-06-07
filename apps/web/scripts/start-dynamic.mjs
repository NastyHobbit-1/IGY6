#!/usr/bin/env node
// Dynamic port starter for grok branch web UI.
// If default port in use, automatically switches to next free port and prints the effective local URL.
// Used by "dev" and "start" scripts so the program always comes up on a clear URL.

import { spawn } from 'child_process';
import net from 'net';

const DEFAULT_PORT = parseInt(process.env.PORT || '3000', 10);
const HOST = '0.0.0.0';
const MAX_TRIES = 20;

function isPortFree(port) {
  return new Promise((resolve) => {
    const server = net.createServer();
    server.once('error', () => resolve(false));
    server.once('listening', () => {
      server.close(() => resolve(true));
    });
    server.listen(port, HOST);
  });
}

async function findFreePort(startPort) {
  for (let p = startPort; p < startPort + MAX_TRIES; p++) {
    if (await isPortFree(p)) {
      return p;
    }
  }
  throw new Error(`No free port found in range ${startPort}-${startPort + MAX_TRIES - 1}`);
}

async function main() {
  const mode = process.argv.includes('--start') ? 'start' : 'dev';
  const port = await findFreePort(DEFAULT_PORT);
  const url = `http://127.0.0.1:${port}`;
  console.log(`[grok] Using clear local URL: ${url} (switched from ${DEFAULT_PORT} if needed)`);
  console.log(`[grok] Starting Next.js ${mode} on ${HOST}:${port} ...`);

  const args = [mode, '--hostname', HOST, '--port', String(port)];
  const child = spawn('next', args, { stdio: 'inherit', shell: false });

  child.on('exit', (code) => process.exit(code || 0));
  child.on('error', (err) => {
    console.error('[grok] Failed to start Next.js:', err);
    process.exit(1);
  });
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
