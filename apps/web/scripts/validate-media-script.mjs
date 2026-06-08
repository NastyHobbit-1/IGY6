import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const page = readFileSync(join(root, "../src/app/page.tsx"), "utf8");
const start = page.indexOf("function MediaImportMvp");
const scriptStart = page.indexOf("const script = `", start);
const marker = "`;\n\n  return (\n    <section className=\"guidedManualText\" id=\"media-import\"";
const scriptEnd = page.indexOf(marker, scriptStart);
const script = page.slice(scriptStart + "const script = `".length, scriptEnd);
writeFileSync(join(root, "_media-check.js"), script);
execSync(`node --check "${join(root, "_media-check.js")}"`, { stdio: "inherit" });
console.log("media script OK", script.split("\n").length, "lines");