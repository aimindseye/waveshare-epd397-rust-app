#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

failed=0
check() {
  local label="$1"
  shift
  if "$@"; then
    printf '%s=ok\n' "$label"
  else
    printf '%s=failed\n' "$label" >&2
    failed=1
  fi
}
contains() {
  local path="$1"
  local pattern="$2"
  grep -Fq -- "$pattern" "$path"
}
not_contains() {
  local path="$1"
  local pattern="$2"
  ! grep -Fq -- "$pattern" "$path"
}

check cargo-version-v0.15.0 grep -Eq '^version = "0\.15\.0"$' Cargo.toml
check sdkconfig-version-v0.15.0 contains sdkconfig.defaults 'CONFIG_APP_PROJECT_VER="0.15.0"'
check milestone-unit-converter-foundation contains src/build_info.rs 'UI_SHELL_MILESTONE: &str = "unit-converter-foundation"'
check calendar-module-exported contains src/lib.rs 'pub mod calendar;'
check unit-converter-module-exported contains src/lib.rs 'pub mod unit_converter;'
check power-key-module-exported contains src/lib.rs 'pub mod power_key;'
check unit-converter-tools-menu-ready contains src/app/menu.rs 'subtitle: "Offline fixed-point conversions"'
check calendar-productivity-menu-ready contains src/app/menu.rs 'subtitle: "Read-only RTC-localized month view"'
check home-dashboard-marker contains src/main.rs 'rustmix-wave=home-dashboard-redesign-ready'
check weather-retry-marker contains src/main.rs 'rustmix-wave=weather-fetch-resilience-ready retries=3 backoff-seconds=2,5,15'
check wake-guard-marker contains src/main.rs 'rustmix-wave=power-key-sleep-entry-wake-guard-ready'
check random-sleep-marker contains src/main.rs 'rustmix-wave=random-sleep-image-selection-ready'
check network-suspended-sleep-marker contains src/main.rs 'rustmix-wave=network-suspended-sleep-image-mode-ready'
check display-config-contract contains src/app/display.rs '/sdcard/RUSTMIX/DISPLAY.TXT'
check weather-config-contract contains src/weather_config.rs '/sdcard/RUSTMIX/WEATHER.TXT'
check wifi-config-contract contains src/network_config.rs '/sdcard/RUSTMIX/WIFI.TXT'
check alarms-config-contract contains src/alarm.rs '/sdcard/RUSTMIX/ALARMS.TXT'
check sleep-image-contract contains src/sleep_images.rs '/sdcard/RUSTMIX/SLEEP'
check raw-font-files-absent bash -c '! find . -type f \( -iname "*.ttf" -o -iname "*.otf" -o -iname "*.woff" -o -iname "*.woff2" \) -print -quit | grep -q .'
check overlay-folders-absent bash -c '! find . -maxdepth 1 -type d -name "v0.*" -print -quit | grep -q .'
check overlay-scripts-absent bash -c '! find scripts -maxdepth 1 -type f \( -name "apply_waveshare_epd397_rust_*" -o -name "validate_waveshare_epd397_rust_*" -o -name "run_waveshare_epd397_rust_*" \) -print -quit | grep -q .'
check local-cache-artifacts-absent bash -c '! find . \( -name ".DS_Store" -o -name "__pycache__" -o -name "*.pyc" \) -print -quit | grep -q .'
check patch-scratch-artifacts-absent bash -c '[ ! -e MANIFEST.txt ] && [ ! -e README-APPLY.md ] && [ ! -e rustmix-sleep-selector-context.txt ] && [ ! -e tests ]'
check example-wifi-password-redacted contains examples/sd-card/RUSTMIX/WIFI.TXT.example 'password=YOUR_PASSWORD'

for script in scripts/*.sh; do
  check "bash-syntax-${script##*/}" bash -n "$script"
done

check rust-lexical-delimiter-scan python3 - <<'PY'
from pathlib import Path
pairs = {')': '(', ']': '[', '}': '{'}

def ok(path: Path) -> bool:
    source = path.read_text(errors='replace')
    stack = []
    i = 0
    block = 0
    while i < len(source):
        if block:
            if source.startswith('/*', i):
                block += 1; i += 2; continue
            if source.startswith('*/', i):
                block -= 1; i += 2; continue
            i += 1; continue
        if source.startswith('//', i):
            end = source.find('\n', i + 2)
            i = len(source) if end < 0 else end + 1
            continue
        if source.startswith('/*', i):
            block = 1; i += 2; continue
        raw = i + 2 if source.startswith('br', i) else i + 1 if source.startswith('r', i) else -1
        if raw >= 0:
            hashes = 0
            while raw < len(source) and source[raw] == '#':
                hashes += 1; raw += 1
            if raw < len(source) and source[raw] == '"':
                delimiter = '"' + '#' * hashes
                end = source.find(delimiter, raw + 1)
                if end < 0:
                    return False
                i = end + len(delimiter); continue
        if source[i] == '"' or (source[i] == 'b' and i + 1 < len(source) and source[i + 1] == '"'):
            quote = i + 1 if source[i] == 'b' else i
            i = quote + 1
            while i < len(source):
                if source[i] == '\\':
                    i += 2; continue
                if source[i] == '"':
                    i += 1; break
                i += 1
            continue
        if source[i] == "'":
            if i + 2 < len(source) and source[i + 2] == "'":
                i += 3; continue
            if i + 3 < len(source) and source[i + 1] == '\\' and source[i + 3] == "'":
                i += 4; continue
            i += 1; continue
        if source[i] in '([{':
            stack.append(source[i])
        elif source[i] in pairs:
            if not stack or stack.pop() != pairs[source[i]]:
                return False
        i += 1
    return not block and not stack

files = list(Path('src').rglob('*.rs'))
raise SystemExit(0 if files and all(ok(path) for path in files) else 1)
PY

if (( failed )); then
  echo 'source-contract-validation=failed' >&2
  exit 1
fi

echo 'source-contract-validation=ok'
