import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const page = readFileSync(join(root, "../src/app/components/UnifiedChatHub.tsx"), "utf8");
const start = page.indexOf("function UnifiedChatHub");
const scriptStart = page.indexOf("const script = `", start);
const marker = "`;\n\n  return (\n    <section\n      className=\"unifiedChatHub\"";
const scriptEnd = page.indexOf(marker, scriptStart);
console.log({ scriptStart, scriptEnd, markerFound: scriptEnd >= 0 });
if (scriptStart < 0 || scriptEnd < 0) {
  console.error("UnifiedChatHub inline script bounds not found");
  process.exit(1);
}
const script = page
  .slice(scriptStart + "const script = `".length, scriptEnd)
  .replace(/\$\{[A-Z0-9_]+\}/g, "0")
  .replaceAll("\\\\", "\\");
const out = join(root, "_chat-check.js");
writeFileSync(out, script);
execSync(`node --check "${out}"`, { stdio: "inherit" });
console.log("chat bounds OK", script.split("\n").length, "lines");
