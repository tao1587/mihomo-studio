import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];

function walk(directory, extensions) {
  if (!fs.existsSync(directory)) return [];
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return walk(target, extensions);
    return extensions.some((extension) => entry.name.endsWith(extension)) ? [target] : [];
  });
}

function relative(file) {
  return path.relative(root, file).split(path.sep).join("/");
}

function forbid(file, content, patterns, layer) {
  for (const pattern of patterns) {
    if (pattern.regex.test(content)) {
      errors.push(`${relative(file)}: ${layer} must not depend on ${pattern.label}`);
    }
  }
}

const rustRoot = path.join(root, "src-tauri", "src");
const rustFiles = walk(rustRoot, [".rs"]);
for (const file of rustFiles) {
  const rel = relative(file);
  const content = fs.readFileSync(file, "utf8");
  if (rel.startsWith("src-tauri/src/domain/")) {
    forbid(file, content, [
      { regex: /crate::application/, label: "application" },
      { regex: /crate::infrastructure/, label: "infrastructure" },
      { regex: /crate::interface/, label: "interface" },
      { regex: /\btauri::/, label: "Tauri" },
      { regex: /\breqwest::/, label: "reqwest" },
    ], "domain");
  } else if (rel.startsWith("src-tauri/src/application/")) {
    forbid(file, content, [
      { regex: /crate::infrastructure/, label: "infrastructure" },
      { regex: /crate::interface/, label: "interface" },
      { regex: /\btauri::/, label: "Tauri" },
      { regex: /\breqwest::/, label: "reqwest" },
    ], "application");
  } else if (rel.startsWith("src-tauri/src/infrastructure/")) {
    forbid(file, content, [
      { regex: /crate::interface/, label: "interface" },
      { regex: /\btauri::/, label: "Tauri" },
    ], "infrastructure");
  } else if (rel.startsWith("src-tauri/src/interface/")) {
    forbid(file, content, [
      { regex: /crate::infrastructure/, label: "infrastructure" },
      { regex: /\breqwest::/, label: "reqwest" },
    ], "interface");
  }
}

const libPath = path.join(rustRoot, "lib.rs");
const lib = fs.readFileSync(libPath, "utf8");
if (/\#\[tauri::command\]/.test(lib)) {
  errors.push("src-tauri/src/lib.rs: composition root must not define Tauri commands");
}

const frontendRoot = path.join(root, "src");
const frontendFiles = walk(frontendRoot, [".ts", ".tsx"]);
for (const file of frontendFiles) {
  const rel = relative(file);
  const content = fs.readFileSync(file, "utf8");
  const imports = [...content.matchAll(/(?:from\s+|import\s*\()\s*["']([^"']+)["']/g)]
    .map((match) => match[1])
    .filter((specifier) => specifier.startsWith("."))
    .map((specifier) => path.resolve(path.dirname(file), specifier));

  for (const target of imports) {
    const targetRel = relative(target);
    if (rel.startsWith("src/shared/") && /src\/(app|features)\//.test(targetRel)) {
      errors.push(`${rel}: shared must not depend on ${targetRel}`);
    }
    if (rel.startsWith("src/features/") && targetRel.startsWith("src/app/")) {
      errors.push(`${rel}: feature must not depend on app (${targetRel})`);
    }
    if (rel.startsWith("src/features/")) {
      const ownFeature = rel.split("/")[2];
      const targetParts = targetRel.split("/");
      if (targetParts[1] === "features" && targetParts[2] !== ownFeature) {
        errors.push(`${rel}: feature slice must not depend on ${targetRel}`);
      }
    }
  }
}

for (const obsolete of [
  "src/App.tsx",
  "src/types.ts",
  "src/domain/catalog.ts",
  "src-tauri/src/catalog.rs",
  "src-tauri/src/profile.rs",
  "src-tauri/src/source",
]) {
  if (fs.existsSync(path.join(root, obsolete))) {
    errors.push(`${obsolete}: obsolete pre-DDD path must be removed`);
  }
}

if (errors.length > 0) {
  console.error("Architecture check failed:\n" + errors.map((error) => `- ${error}`).join("\n"));
  process.exit(1);
}

console.log(
  `Architecture check passed: ${rustFiles.length} Rust files, ${frontendFiles.length} frontend files.`,
);
