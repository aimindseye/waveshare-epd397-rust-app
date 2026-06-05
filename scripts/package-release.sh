#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

./scripts/validate_source_contract.sh

VERSION="$(sed -n 's/^version = "\([^"]*\)"/\1/p' Cargo.toml | head -n1)"
if [[ -z "$VERSION" ]]; then
  echo 'Unable to determine Cargo package version.' >&2
  exit 1
fi

mkdir -p dist
OUT="dist/waveshare-epd397-rust-app-v${VERSION}-github-ready.zip"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/waveshare-epd397-rust-app"
rsync -a ./ "$TMP/waveshare-epd397-rust-app/" \
  --exclude '.git/' \
  --exclude 'target/' \
  --exclude '.embuild/' \
  --exclude 'dist/' \
  --exclude '.DS_Store' \
  --exclude '__pycache__/' \
  --exclude '*.pyc' \
  --exclude '*.bak' \
  --exclude '*.orig' \
  --exclude '*.rej'

rm -f "$OUT" "$OUT.sha256"
(
  cd "$TMP"
  zip -qr "$ROOT/$OUT" waveshare-epd397-rust-app
)
(
  cd dist
  shasum -a 256 "$(basename "$OUT")" > "$(basename "$OUT").sha256"
)

echo "release-zip=$OUT"
echo "release-sha256=$OUT.sha256"
