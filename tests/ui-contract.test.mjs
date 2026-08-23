// Node built-in test that extends the existing UI smoke harness.
// Verifies product UI contract (selectors, data attributes) without Playwright/browsers.
import { test } from 'node:test';
import { spawn } from 'node:child_process';

test('apps/web ui-smoke passes', async (t) => {
  await new Promise((resolve, reject) => {
    const child = spawn(process.execPath, ['apps/web/scripts/ui-smoke.mjs'], {
      cwd: process.cwd(),
      stdio: ['ignore', 'pipe', 'pipe'],
      env: { ...process.env, FORCE_COLOR: '0' },
    });
    let stderr = '';
    child.stderr.on('data', (d) => { stderr += d.toString(); });
    child.on('exit', (code) => {
      if (code === 0) return resolve();
      reject(new Error(`ui-smoke failed with code ${code}\n${stderr}`));
    });
    child.on('error', reject);
  });
});

