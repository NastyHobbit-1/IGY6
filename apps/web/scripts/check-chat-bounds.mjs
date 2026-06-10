import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const page = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../src/app/page.tsx"), "utf8");
const start = page.indexOf("function UnifiedChatHub");
const scriptStart = page.indexOf("const script = `", start);
const marker = "`;\n\n  return (\n    <section\n      className=\"unifiedChatHub\"";
const scriptEnd = page.indexOf(marker, scriptStart);
console.log({ scriptStart, scriptEnd, markerFound: scriptEnd >= 0 });
const script = page.slice(scriptStart + "const script = `".length, scriptEnd);
writeFileSync(join(dirname(fileURLToPath(import.meta.url)), "_chat-check.js"), script);
execSync("node --check scripts/_chat-check.js", { stdio: "inherit" });
console.log("chat bounds OK", script.split("\n").length, "lines");