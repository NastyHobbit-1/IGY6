import { readdirSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(fileURLToPath(import.meta.url));
const componentsDir = join(root, "../src/app/components");

function listTsx(dir) {
  const out = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) out.push(...listTsx(path));
    else if (entry.name.endsWith(".tsx") || entry.name.endsWith(".ts")) out.push(path);
  }
  return out;
}

const scripts = [];
const panelRe = /const (?:script|grokToolsScript) = `([\s\S]*?)`;\n\n  return \(/g;
const inlineRe = /<ClientScript script=\{`([\s\S]*?)`\}/g;

for (const file of listTsx(componentsDir)) {
  const source = readFileSync(file, "utf8");
  let match;
  const panel = new RegExp(panelRe.source, "g");
  const inline = new RegExp(inlineRe.source, "g");
  while ((match = panel.exec(source)) !== null) {
    scripts.push({ kind: "panel", file, source: match[1] });
  }
  while ((match = inline.exec(source)) !== null) {
    scripts.push({ kind: "inline", file, source: match[1] });
  }
}

let failures = 0;
for (const [index, item] of scripts.entries()) {
  try {
    new Function(item.source.replace(/\$\{[A-Z0-9_]+\}/g, "0").replaceAll("\\\\", "\\"));
    console.log(`#${index} ${item.kind} OK (${item.source.split("\n").length} lines) ${item.file.split("/components/").pop()}`);
  } catch (error) {
    failures += 1;
    console.error(`Script #${index} (${item.kind}) invalid in ${item.file}: ${error.message}`);
    console.error(item.source.slice(0, 240).replace(/\n/g, "\\n"));
    console.error("---");
  }
}

if (failures > 0) {
  process.exit(1);
}
console.log(`Validated ${scripts.length} panel scripts (${failures} failures).`);
