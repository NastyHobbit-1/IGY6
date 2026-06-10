import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const page = readFileSync(join(root, "../src/app/page.tsx"), "utf8");
const start = page.indexOf("function UnifiedChatHub");
const scriptStart = page.indexOf("const script = `", start);
const scriptEnd = page.indexOf('`\n\n  return (\n    <section\n      className="unifiedChatHub"', scriptStart);
const script = page.slice(scriptStart + "const script = `".length, scriptEnd);
const out = join(root, "_chat-check.js");
writeFileSync(out, script, "utf8");

try {
  execSync(`node --check "${out}"`, { stdio: "pipe" });
  console.log("chat script OK", script.length);
} catch (error) {
  const detail = error.stderr?.toString() || error.message;
  console.error("chat script BAD:\n", detail);
  const lines = script.split("\n");
  for (const [index, line] of lines.entries()) {
    if (line.includes("Ollama") || line.includes("add data")) {
      console.error(`${index + 1}: ${line}`);
    }
  }
  process.exit(1);
}