// Fails when an en dash (U+2013) or em dash (U+2014) appears in tracked text files.
// The project uses plain hyphens everywhere: code, interface text and documentation.

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

// Built from code points so this file does not contain the characters it bans.
const BANNED = new RegExp(`[${String.fromCodePoint(0x2013, 0x2014)}]`, "u");
const BINARY = /\.(png|ico|icns|bmp|jpg|jpeg|gif|webp|woff2?)$/i;

const files = execFileSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], {
  encoding: "utf8",
})
  .split("\n")
  .filter((file) => file && !BINARY.test(file) && !file.endsWith("package-lock.json"));

const hits = [];
for (const file of files) {
  let text;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    continue;
  }
  text.split("\n").forEach((line, index) => {
    if (BANNED.test(line)) hits.push(`${file}:${index + 1}: ${line.trim().slice(0, 120)}`);
  });
}

if (hits.length > 0) {
  console.error(`Found ${hits.length} line(s) with an en or em dash; use a plain hyphen:\n`);
  console.error(hits.join("\n"));
  process.exit(1);
}
console.log(`No en or em dashes in ${files.length} files.`);
