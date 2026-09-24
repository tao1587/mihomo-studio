import { readFile } from "node:fs/promises";

const readWorkflow = async (path) => {
  try {
    return await readFile(path, "utf8");
  } catch (error) {
    throw new Error(`Missing required workflow: ${path}`, { cause: error });
  }
};

const requireMatch = (content, pattern, message) => {
  if (!pattern.test(content)) {
    throw new Error(message);
  }
};

const requirePinnedActions = (content, path) => {
  const actionRefs = [...content.matchAll(/uses:\s+[^\s@]+@([^\s#]+)/g)];
  if (actionRefs.length === 0) {
    throw new Error(`${path} does not use any actions`);
  }

  for (const [, ref] of actionRefs) {
    if (!/^[0-9a-f]{40}$/.test(ref)) {
      throw new Error(`${path} contains an action that is not pinned to a full commit SHA`);
    }
  }
};

const ciPath = ".github/workflows/ci.yml";
const releasePath = ".github/workflows/release.yml";
const ci = await readWorkflow(ciPath);
const release = await readWorkflow(releasePath);

requireMatch(ci, /pull_request:/, "CI must run for pull requests");
requireMatch(ci, /push:/, "CI must run for pushes");
requireMatch(ci, /permissions:\s*\n\s+contents:\s+read/, "CI must use read-only contents permission");
for (const command of [
  "npm run check:spec",
  "npm test",
  "npm run build",
  "cargo fmt --check",
  "cargo test",
  "cargo check",
]) {
  if (!ci.includes(command)) {
    throw new Error(`CI is missing command: ${command}`);
  }
}
requirePinnedActions(ci, ciPath);

requireMatch(release, /tags:\s*\n\s+- ['"]v\*['"]/, "Release must only run for v* tags");
requireMatch(
  release,
  /permissions:\s*\n\s+contents:\s+write/,
  "Release must declare contents write permission",
);
for (const platform of ["macos-latest", "ubuntu-22.04", "windows-latest"]) {
  if (!release.includes(platform)) {
    throw new Error(`Release matrix is missing ${platform}`);
  }
}
requireMatch(release, /tauri-apps\/tauri-action@[0-9a-f]{40}/, "Release must use pinned tauri-action");
requireMatch(release, /tagName:\s*\$\{\{ github\.ref_name \}\}/, "Release must use the pushed tag");
requireMatch(release, /releaseDraft:\s*false/, "Release must publish a non-draft release");
requireMatch(release, /generateReleaseNotes:\s*true/, "Release must generate release notes");
requirePinnedActions(release, releasePath);

console.log("GitHub workflow checks passed: CI and three-platform tagged releases are configured.");
