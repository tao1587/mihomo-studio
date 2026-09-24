#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
manifest="$root/src-tauri/sidecars/mihomo-v1.19.29.json"
golden_dir="$root/src-tauri/src/domain/compile/fixtures"

if [[ "$(uname -s)" != "Darwin" || "$(uname -m)" != "arm64" ]]; then
  echo "Pinned fixture validation currently supports darwin-arm64 only." >&2
  exit 2
fi

metadata="$({ node -e '
  const fs = require("fs");
  const value = JSON.parse(fs.readFileSync(process.argv[1], "utf8"));
  const artifact = value.artifacts["darwin-arm64"];
  process.stdout.write([value.version, artifact.url, artifact.compressedSha256].join("\t"));
' "$manifest"; })"
IFS=$'\t' read -r version url expected_sha <<< "$metadata"

workdir="$(mktemp -d "${TMPDIR:-/tmp}/mihomo-studio-validation.XXXXXX")"
trap 'rm -rf "$workdir"' EXIT

archive="$workdir/mihomo.gz"
binary="$workdir/mihomo"
home="$workdir/home"
mkdir -p "$home"

curl -L -sS "$url" -o "$archive"
actual_sha="$(shasum -a 256 "$archive" | awk '{print $1}')"
if [[ "$actual_sha" != "$expected_sha" ]]; then
  echo "Mihomo archive SHA-256 mismatch for $version." >&2
  exit 1
fi

gunzip -c "$archive" > "$binary"
chmod 700 "$binary"
for golden in \
  "$golden_dir/strict-profile.golden.yml" \
  "$golden_dir/strict-vless-profile.golden.yml" \
  "$golden_dir/strict-domain-profile.golden.yml" \
  "$golden_dir/wireguard-node.golden.yml"
do
  cp "$golden" "$home/config.yaml"
  "$binary" -t -d "$home" -f "$home/config.yaml"
done
