import { chromium } from "playwright";

const browser = await chromium.launch({ headless: true });
const page = await browser.newPage();
const errors = [];
page.on("pageerror", (err) => errors.push(String(err)));

await page.goto("http://127.0.0.1:3002/", { waitUntil: "domcontentloaded", timeout: 120000 });
await page.waitForTimeout(5000);

const info = await page.evaluate(() => ({
  unifiedChat: Boolean(document.querySelector("[data-unified-chat]")),
  chatWired: document.querySelector("[data-unified-chat]")?.getAttribute("data-chat-wired") ?? null,
  settingsEnv: Boolean(document.querySelector("[data-settings-env]")),
  clientScripts: document.querySelectorAll("body > script").length,
  tabResults: document.getElementById("tab-results")?.checked ?? null
}));

console.log(JSON.stringify({ info, errors }, null, 2));
await browser.close();