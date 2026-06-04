#!/usr/bin/env bash
set -euo pipefail

PORT="${1:-}"
BIN="target/xtensa-esp32s3-espidf/release/waveshare-epd397-rust-app"

cargo +esp build --release

if [[ -n "$PORT" ]]; then
  exec espflash flash --chip esp32s3 --port "$PORT" --monitor "$BIN"
else
  exec espflash flash --chip esp32s3 --monitor "$BIN"
fi
