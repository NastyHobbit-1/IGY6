import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const page = readFileSync(join(root, "../src/app/components/MediaImportMvp.tsx"), "utf8");
const start = page.indexOf("function MediaImportMvp");
const scriptStart = page.indexOf("const script = `", start);
const marker = "`;\n\n  return (\n    <section className=\"guidedManualText\" id=\"media-import\"";
const scriptEnd = page.indexOf(marker, scriptStart);
if (scriptStart < 0 || scriptEnd < 0) {
  console.error("MediaImportMvp inline script bounds not found");
  process.exit(1);
}
const script = page.slice(scriptStart + "const script = `".length, scriptEnd);
const out = join(root, "_media-check.js");
writeFileSync(out, script);
execSync(`node --check "${out}"`, { stdio: "inherit" });
console.log("media script OK", script.split("\n").length, "lines");
