import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const page = readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../src/app/page.tsx"), "utf8");
const scripts = [];
const panelRe = /const (?:script|grokToolsScript) = `([\s\S]*?)`;\n\n  return \(/g;
const inlineRe = /<ClientScript script=\{`([\s\S]*?)`\}/g;

let match;
while ((match = panelRe.exec(page)) !== null) {
  scripts.push({ kind: "panel", source: match[1] });
}
while ((match = inlineRe.exec(page)) !== null) {
  scripts.push({ kind: "inline", source: match[1] });
}

let failures = 0;
for (const [index, item] of scripts.entries()) {
  try {
    new Function(item.source);
    console.log(`#${index} ${item.kind} OK (${item.source.split("\n").length} lines)`);
  } catch (error) {
    failures += 1;
    console.error(`Script #${index} (${item.kind}) invalid: ${error.message}`);
    console.error(item.source.slice(0, 240).replace(/\n/g, "\\n"));
    console.error("---");
  }
}

if (failures > 0) {
  process.exit(1);
}
console.log(`Validated ${scripts.length} panel scripts (${failures} failures).`);