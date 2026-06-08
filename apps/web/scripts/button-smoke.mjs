import { chromium } from "playwright";

const baseUrl = process.env.UI_URL ?? "http://127.0.0.1:3002";

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (err) => errors.push(String(err)));
page.on("console", (msg) => {
  if (msg.type() === "error") errors.push(msg.text());
});

await page.goto(baseUrl, { waitUntil: "domcontentloaded", timeout: 120000 });
await page.waitForFunction(() => document.querySelector("[data-unified-chat]"), { timeout: 60000 });
await page.waitForFunction(
  () => document.querySelector("[data-unified-chat]")?.getAttribute("data-chat-wired") === "true",
  { timeout: 30000 }
).catch(() => null);

const chatWired = await page.evaluate(() => {
  const hub = document.querySelector("[data-unified-chat]");
  return hub?.getAttribute("data-chat-wired") ?? "missing-hub";
});

await page.getByRole("tab", { name: "Settings" }).click();
await page.waitForTimeout(500);

const settingsWired = await page.evaluate(() => {
  const root = document.querySelector("[data-settings-env]");
  return root?.getAttribute("data-settings-wired") ?? "missing-root";
});

let authAlert = null;
page.on("dialog", async (dialog) => {
  authAlert = dialog.message();
  await dialog.dismiss();
});

await page.getByRole("button", { name: "Auth status" }).click();
await page.waitForTimeout(1000);

let verifyClicked = false;
await page.evaluate(() => {
  const btn = document.querySelector("[data-settings-verify]");
  if (btn) {
    btn.click();
    window.__verifyClicked = true;
  }
});
verifyClicked = await page.evaluate(() => Boolean(window.__verifyClicked));

const resultText = await page.locator("[data-settings-result]").textContent();

console.log(JSON.stringify({
  baseUrl,
  chatWired,
  settingsWired,
  authAlert: authAlert?.slice(0, 120) ?? null,
  verifyClicked,
  resultText: resultText?.slice(0, 120) ?? null,
  errors: errors.slice(0, 10)
}, null, 2));

await browser.close();
process.exit(errors.length > 0 || chatWired !== "true" || settingsWired !== "true" ? 1 : 0);