import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const errors = [];
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const readJson = (file) => JSON.parse(read(file));
const sorted = (values) => [...values].sort();

function expectEqual(label, actual, expected) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    errors.push(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function collectMarkdownFiles(directory) {
  if (!fs.existsSync(directory)) return [];
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) return collectMarkdownFiles(target);
    return entry.name.endsWith(".md") ? [target] : [];
  });
}

const index = readJson("openspec/spec-index.json");
const packageJson = readJson("package.json");
const tauriConfig = readJson("src-tauri/tauri.conf.json");
const cargoVersion = read("src-tauri/Cargo.toml").match(/^version\s*=\s*"([^"]+)"/m)?.[1];
expectEqual("version sources", [packageJson.version, cargoVersion, tauriConfig.version], [
  packageJson.version,
  packageJson.version,
  packageJson.version,
]);

const compileContract = index.contracts?.compileProfileRequest;
if (!compileContract) {
  errors.push("spec-index.json: missing compileProfileRequest contract");
} else {
  const frontendContract = read(compileContract.frontendPath);
  const backendContract = read(compileContract.backendPath);
  const toSnakeCase = (value) => value.replace(
    /[A-Z]/g,
    (character) => `_${character.toLowerCase()}`,
  );
  for (const field of compileContract.fields) {
    if (!new RegExp(`\\b${field}\\s*:`).test(frontendContract)) {
      errors.push(`compileProfileRequest: frontend field missing ${field}`);
    }
    const backendField = toSnakeCase(field);
    if (!new RegExp(`\\b${backendField}\\s*:`).test(backendContract)) {
      errors.push(`compileProfileRequest: backend field missing ${backendField}`);
    }
  }
}

const capabilityIds = index.capabilities.map((capability) => capability.id);
expectEqual("unique capability ids", new Set(capabilityIds).size, capabilityIds.length);
for (const capability of index.capabilities) {
  for (const target of [capability.specPath, ...capability.implementationPaths]) {
    if (!fs.existsSync(path.join(root, target))) {
      errors.push(`${capability.id}: missing mapped path ${target}`);
    }
  }
  if (fs.existsSync(path.join(root, capability.specPath))) {
    const spec = read(capability.specPath);
    if (!spec.includes("## Requirements")) {
      errors.push(`${capability.specPath}: missing Requirements section`);
    }
    if (!/## (Planned )?Implementation Map/.test(spec)) {
      errors.push(`${capability.specPath}: missing Implementation Map section`);
    }
    if (!/## (Required )?Verification/.test(spec)) {
      errors.push(`${capability.specPath}: missing Verification section`);
    }
  }
}

const lib = read("src-tauri/src/lib.rs");
const rustModules = [...lib.matchAll(/^mod\s+([a-zA-Z0-9_]+);/gm)].map((match) => match[1]);
expectEqual(
  "Rust composition modules",
  sorted(rustModules),
  sorted(index.architecture.expectedRustModules),
);
const handlerBody = lib.match(/tauri::generate_handler!\[([\s\S]*?)\]/)?.[1] ?? "";
const registeredCommands = [...handlerBody.matchAll(/::([a-zA-Z0-9_]+)\s*,/g)]
  .map((match) => match[1]);
expectEqual(
  "registered IPC commands",
  sorted(registeredCommands),
  sorted(index.architecture.expectedIpcCommands),
);

const frontendFiles = [];
function collectFrontend(directory) {
  for (const entry of fs.readdirSync(directory, { withFileTypes: true })) {
    const target = path.join(directory, entry.name);
    if (entry.isDirectory()) collectFrontend(target);
    else if (/\.tsx?$/.test(entry.name)) frontendFiles.push(target);
  }
}
collectFrontend(path.join(root, "src"));
const invokedCommands = new Set();
for (const file of frontendFiles) {
  const content = fs.readFileSync(file, "utf8");
  for (const match of content.matchAll(/invoke(?:<[^>]+>)?\(\s*["']([^"']+)["']/g)) {
    invokedCommands.add(match[1]);
  }
}
expectEqual(
  "frontend IPC commands",
  sorted(invokedCommands),
  sorted(index.architecture.expectedIpcCommands),
);

const catalog = readJson(index.catalog.path);
expectEqual("catalog schema", catalog.schemaVersion, index.catalog.schemaVersion);
expectEqual("catalog source count", catalog.sources.length, index.catalog.sourceCount);
expectEqual("catalog rule-set count", catalog.ruleSets.length, index.catalog.ruleSetCount);
expectEqual(
  "simple default sources",
  sorted(catalog.sources
    .filter((source) => source.defaultEnabledModes.includes("simple"))
    .map((source) => source.id)),
  sorted(index.catalog.simpleDefaultSourceIds),
);
const finalRules = catalog.ruleSets.filter(
  (ruleSet) => ruleSet.id === index.catalog.finalRuleSetId
    && ruleSet.defaultEnabledModes.includes("simple"),
);
expectEqual("simple final rule count", finalRules.length, 1);
if (finalRules.length === 1) {
  expectEqual("final rule order", finalRules[0].order, index.catalog.finalRuleOrder);
  const maxDefaultOrder = Math.max(...catalog.ruleSets
    .filter((ruleSet) => ruleSet.defaultEnabledModes.includes("simple"))
    .map((ruleSet) => ruleSet.order));
  expectEqual("final rule is last", finalRules[0].order, maxDefaultOrder);
}

const markdownFiles = [
  path.join(root, "README.md"),
  path.join(root, "docs", "README.md"),
  ...collectMarkdownFiles(path.join(root, "openspec")),
];
for (const file of markdownFiles) {
  const content = fs.readFileSync(file, "utf8");
  for (const match of content.matchAll(/\[[^\]]+\]\(([^)]+)\)/g)) {
    const href = match[1].trim();
    if (/^(https?:|mailto:|#)/.test(href)) continue;
    const targetPart = href.split("#")[0];
    if (!targetPart) continue;
    const target = path.resolve(path.dirname(file), decodeURIComponent(targetPart));
    if (!fs.existsSync(target)) {
      errors.push(`${path.relative(root, file)}: broken local link ${href}`);
    }
  }
}

if (!read("README.md").includes("openspec/project.md")) {
  errors.push("README.md: missing OpenSpec entrypoint");
}

if (errors.length > 0) {
  console.error("Spec drift check failed:\n" + errors.map((error) => `- ${error}`).join("\n"));
  process.exit(1);
}

console.log(
  `Spec drift check passed: ${index.capabilities.length} capabilities, ${registeredCommands.length} IPC commands, ${catalog.sources.length} sources, ${catalog.ruleSets.length} rule sets.`,
);
