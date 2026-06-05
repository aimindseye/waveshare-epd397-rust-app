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

check cargo-version-v0.16.8 grep -Eq '^version = "0\.16\.8"$' Cargo.toml
check sdkconfig-version-v0.16.8 contains sdkconfig.defaults 'CONFIG_APP_PROJECT_VER="0.16.8"'
check milestone-library-bookmark-tab-rendering-alignment contains src/build_info.rs 'UI_SHELL_MILESTONE: &str = "library-bookmark-tab-rendering-alignment"'
check calendar-module-exported contains src/lib.rs 'pub mod calendar;'
check reader-module-exported contains src/lib.rs 'pub mod reader;'
check reader-screen-exported contains src/app/screens/mod.rs 'pub mod reader;'
check reader-typography-exported contains src/app/mod.rs 'pub mod reader_typography;'
check reader-serif-assets-exported contains src/app/mod.rs 'pub mod reader_serif_assets;'
check reader-menu-all-ready contains src/app/menu.rs 'subtitle: "Resume the last saved book"'
check reader-menu-bookmarks-ready contains src/app/menu.rs 'subtitle: "Saved reading positions"'
check reader-books-contract contains src/reader.rs '/sdcard/RUSTMIX/BOOKS'
check reader-staged-open-contract contains src/reader.rs 'ReaderLoadingStage'
check reader-first-page-first-contract contains src/reader.rs 'FirstPageReady'
check reader-nearby-cache-contract contains src/reader.rs 'READER_NEARBY_PAGE_CACHE'
check reader-toc-shell-contract contains src/reader.rs 'TableOfContents'
check reader-clear-ghost-marker contains src/main.rs 'rustmix-wave=reader-clear-ghosting refresh=global-base'
check reader-state-directory-contract contains src/reader.rs '/sdcard/RUSTMIX/READER'
check reader-state-file-contract contains src/reader.rs 'READER_STATE_FILE: &str = "STATE.TXT"'
check reader-recent-file-contract contains src/reader.rs 'READER_RECENT_FILE: &str = "RECENT.TXT"'
check reader-bookmarks-file-contract contains src/reader.rs 'READER_BOOKMARKS_FILE: &str = "MARKS.TXT"'
check reader-preferences-file-contract contains src/reader.rs 'READER_PREFS_FILE: &str = "PREFS.TXT"'
check reader-anchor-cache-contract contains src/reader.rs 'READER_CACHE_DIRECTORY: &str = "CACHE"'
check reader-atomic-replace-contract contains src/reader.rs 'fn atomic_replace_text'
check reader-backup-fallback-contract contains src/reader.rs 'load_with_backup'
check reader-cache-fingerprint-contract contains src/reader.rs 'layout.book_font.marker()'
check reader-bookmark-toggle-contract contains src/reader.rs 'toggle_current_bookmark'
check reader-normalization-contract contains src/reader.rs 'fn normalize_decoded'
check reader-emphasis-normalization-contract contains src/reader.rs "character == '_'"
check reader-multiline-emphasis-contract contains src/reader.rs 'repeated_separator'
check reader-word-internal-underscore-contract contains src/reader.rs 'word_internal'
check reader-shared-body-geometry-contract contains src/app/screens/reader.rs 'ReaderBodyGeometry'
check reader-text-bounds-contract contains src/app/typography/mod.rs 'pub struct TextBounds'
check reader-clipped-draw-contract contains src/app/typography/mod.rs 'pub fn draw_clipped'
check reader-theme-contract contains src/reader.rs 'pub enum ReadingTheme'
check reader-orientation-contract contains src/reader.rs 'pub enum ReaderOrientation'
check reader-font-size-xlarge-contract contains src/reader.rs 'XLarge'
check reader-serif-font-contract contains src/reader.rs 'Self::Serif => "Serif"'
check reader-show-progress-contract contains src/reader.rs 'show_progress: bool'
check reader-layout-rebuild-contract contains src/reader.rs 'request_layout_rebuild'
check reader-serif-raster-contract contains src/app/reader_serif_assets.rs 'pub static SERIF_XLARGE'
check reader-typography-resolver-contract contains src/app/reader_typography.rs 'reader_body_style'
check reader-persistence-marker contains src/main.rs 'rustmix-wave=reader-state-persistence-ready'
check reader-bookmarks-marker contains src/main.rs 'rustmix-wave=reader-bookmarks-ready'
check reader-ux-repair-marker contains src/main.rs 'rustmix-wave=reader-ux-repair-ready'
check reader-preferences-marker contains src/main.rs 'rustmix-wave=reader-preferences-ready'
check reader-high-contrast-layout-marker contains src/main.rs 'rustmix-wave=reader-high-contrast-layout-ready'
check reader-txt-emphasis-cleanup-marker contains src/main.rs 'rustmix-wave=reader-txt-emphasis-cleanup-ready'
check reader-positions-file-contract contains src/reader.rs 'READER_POSITIONS_FILE: &str = "POSITS.TXT"'
check reader-position-limit-contract contains src/reader.rs 'READER_POSITION_LIMIT: usize = 64'
check reader-per-book-resume-contract contains src/reader.rs 'saved_position_for_book'
check reader-preferences-editor-contract contains src/reader.rs 'pub enum ReadingPreference'
check reader-paragraph-alignment-contract contains src/reader.rs 'pub enum ParagraphAlignment'
check reader-paragraph-alignment-default contains src/reader.rs 'paragraph_alignment: ParagraphAlignment::Justified'
check reader-paragraph-cache-fingerprint contains src/reader.rs 'layout.paragraph_alignment.marker()'
check reader-preferences-route contains src/app/router.rs 'ReaderPreferences'
check reader-preferences-renderer contains src/app/screens/reader.rs 'pub fn render_preferences'
check reader-per-book-marker contains src/main.rs 'rustmix-wave=reader-per-book-resume-ready'
check reader-controls-marker contains src/main.rs 'rustmix-wave=reader-controls-alignment-ready'
check reader-options-split-marker contains src/main.rs 'rustmix-wave=reader-options-split-ready'
check reader-preferences-settings-navigation-marker contains src/main.rs 'rustmix-wave=reader-preferences-settings-navigation-ready'
check reader-preferences-settings-input-policy contains src/app/state.rs 'self.reader.cycle_preference_previous()'
check reader-preferences-select-change-policy contains src/app/state.rs 'self.reader.activate_selected_preference()'
check reader-preferences-settings-navigation-doc test -f docs/READER_PREFERENCES_SETTINGS_NAVIGATION.md
check reader-v0.16.5-smoke-test test -f docs/V0.16.5-PHYSICAL-SMOKE-TEST.md
check reader-v0.16.6-smoke-test test -f docs/V0.16.6-PHYSICAL-SMOKE-TEST.md
check reader-fat83-doc test -f docs/READER_FAT83_PERSISTENCE.md
check reader-fat83-legacy-read-contract contains src/reader.rs 'LEGACY_READER_POSITIONS_FILE: &str = "POSITIONS.TXT"'
check reader-fat83-cache-basename-contract contains src/reader.rs 'format!("{:08X}.CCH"'
check reader-fat83-runtime-guard contains src/reader.rs 'is_fat83_safe_file_name'
check reader-fat83-old-cache-prefix-write-absent not_contains src/reader.rs '.join(format!("B{:08X}.CCH"'
check reader-fat83-legacy-write-absent not_contains src/reader.rs 'atomic_replace_text(&self.legacy_positions_path'
check reader-fat83-marker contains src/main.rs 'rustmix-wave=reader-fat83-persistence-ready'
check reader-fat83-runtime-marker contains src/main.rs 'rustmix-wave=reader-fat83-runtime-ready'
check reader-bookmark-page-label-marker contains src/main.rs 'rustmix-wave=reader-bookmark-page-labels-ready'
check reader-bookmark-page-label-render contains src/app/screens/reader.rs 'bookmark_display_page(bookmark)'
check reader-cache-file-helper contains src/reader.rs 'fn cache_file_name_for'
check reader-duplicate-persistence-log-suppression contains src/reader.rs 'last_persistence_event'
check reader-v0.16.7-smoke-test test -f docs/V0.16.7-PHYSICAL-SMOKE-TEST.md
check reader-v0.16.7-doc test -f docs/READER_FAT83_RUNTIME_BOOKMARK_LABELS.md
check library-bookmark-tab-marker contains src/main.rs 'rustmix-wave=library-bookmark-tab-rendering-ready'
check library-bookmark-tab-status contains src/app/screens/reader.rs 'middle: format!("{entry_count} saved")'
check library-bookmark-tab-source contains src/app/screens/reader.rs 'right: "MARKS.TXT"'
check library-bookmark-tab-page-columns contains src/app/screens/reader.rs 'badge: "PAGE".into()'
check library-bookmark-tab-page-resolver contains src/app/screens/reader.rs 'reader.bookmark_display_page(bookmark)'
check library-bookmark-tab-epub-note-gated contains src/app/screens/reader.rs 'if reader.library_tab != ReaderLibraryTab::Bookmarks'
check reader-v0.16.8-doc test -f docs/READER_LIBRARY_BOOKMARK_TAB_ALIGNMENT.md
check reader-v0.16.8-smoke-test test -f docs/V0.16.8-PHYSICAL-SMOKE-TEST.md
check reader-v0.16.4-doc test -f docs/READER_PER_BOOK_RESUME_CONTROLS.md
check reader-v0.16.4-smoke-test test -f docs/V0.16.4-PHYSICAL-SMOKE-TEST.md
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
check font-notice-serif contains docs/licenses/FONT_NOTICES.md 'DejaVu Serif'
check reader-prefs-doc test -f docs/READER_UX_PREFERENCES.md
check reader-prefs-example contains examples/sd-card/RUSTMIX/READER/PREFS.TXT.example 'book_font=serif'
check reader-prefs-example-install contains scripts/install-sd-examples.sh 'PREFS.TXT.example'

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
