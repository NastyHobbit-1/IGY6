import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const page = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../src/app/page.tsx"), "utf8");
const start = page.indexOf("function UnifiedChatHub");
const scriptStart = page.indexOf("const script = `", start);
const scriptEnd = page.indexOf('`\n\n  return (\n    <section\n      className="unifiedChatHub"', scriptStart);
const script = page.slice(scriptStart + "const script = `".length, scriptEnd);
const lines = script.split("\n");

for (let i = 1; i <= lines.length; i += 1) {
  const chunk = lines.slice(0, i).join("\n");
  try {
    new Function(chunk);
  } catch (error) {
    if (i === lines.length || !String(error.message).includes("Unexpected")) {
      console.error(`Fails through line ${i}: ${error.message}`);
      console.error(lines[Math.max(0, i - 3)]);
      console.error(lines[Math.max(0, i - 2)]);
      console.error(lines[Math.max(0, i - 1)]);
      break;
    }
  }
}