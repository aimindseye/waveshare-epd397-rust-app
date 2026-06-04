#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FORCE=0
if [[ "${1:-}" == "--force" ]]; then
  FORCE=1
  shift
fi
VOLUME="${1:-}"
if [[ -z "$VOLUME" ]]; then
  echo "usage: $0 [--force] /Volumes/YOUR_SD_CARD" >&2
  exit 1
fi

SRC="$ROOT/examples/sd-card/RUSTMIX"
DEST="$VOLUME/RUSTMIX"
mkdir -p "$DEST/SLEEP"

copy_example() {
  local src="$1"
  local dest="$2"
  if [[ -e "$dest" && "$FORCE" -ne 1 ]]; then
    echo "preserved-existing=$dest"
    return
  fi
  cp "$src" "$dest"
  echo "installed=$dest"
}

copy_example "$SRC/WIFI.TXT.example" "$DEST/WIFI.TXT"
copy_example "$SRC/WEATHER.TXT.example" "$DEST/WEATHER.TXT"
copy_example "$SRC/ALARMS.TXT.example" "$DEST/ALARMS.TXT"
copy_example "$SRC/DISPLAY.TXT.example" "$DEST/DISPLAY.TXT"
copy_example "$SRC/SLEEP/SLEEP.BMP" "$DEST/SLEEP/SLEEP.BMP"
copy_example "$SRC/SLEEP/SLEEP01.BMP" "$DEST/SLEEP/SLEEP01.BMP"

echo "rustmix-wave-sd-examples-ready=$DEST"
