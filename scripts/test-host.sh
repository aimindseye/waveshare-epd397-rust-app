#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

HOST_TRIPLE="$(rustc +stable -vV | sed -n 's/^host: //p')"
if [[ -z "$HOST_TRIPLE" ]]; then
  echo 'Unable to determine the stable Rust host target.' >&2
  exit 1
fi

cargo +stable test --target "$HOST_TRIPLE" --lib
