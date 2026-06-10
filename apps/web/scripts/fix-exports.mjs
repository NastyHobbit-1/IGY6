import fs from "fs";
import path from "path";

const componentsDir = path.resolve("apps/web/src/app/components");

function walk(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (entry.name.endsWith(".tsx")) {
      let content = fs.readFileSync(full, "utf8");
      const fixed = content.replace(/export function (\w+)\1/g, "export function $1");
      if (fixed !== content) {
        fs.writeFileSync(full, fixed);
        console.log("fixed", path.relative(componentsDir, full));
      }
    }
  }
}

walk(componentsDir);