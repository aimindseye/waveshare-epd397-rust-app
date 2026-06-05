# Repository cleanup report

This repository was consolidated from the physical-test v0.16.4 working tree into a GitHub-ready source snapshot.

## Removed

- Incremental overlay replacement directories from v0.13.0 through v0.15.0.
- Overlay apply scripts, overlay validators and overlay self-test runners.
- Patch-only fixture trees under `tests/`.
- Extracted package manifests: `MANIFEST.txt` and `README-APPLY.md`.
- Source-context scratch export: `rustmix-sleep-selector-context.txt`.
- `.DS_Store`, Python bytecode and `__pycache__` artifacts.
- Outdated milestone-by-milestone delivery documents that conflicted with the current state.

## Retained

- Active Rust source modules.
- Cargo metadata and ESP-IDF configuration.
- VS Code workspace settings.
- Maintained build, flash, host-test and SD-install helpers.
- Example removable-SD configuration files and sleep BMPs.
- Font-license text and current font notice.

## Added or repaired

- Current-state README.
- Consolidated CHANGELOG.
- Current architecture, board contract, SD setup, smoke test, known issues and GitHub upload documents.
- Missing `scripts/validate_source_contract.sh` referenced by build and validation helpers.
- `scripts/install-sd-examples.sh`.
- `scripts/package-release.sh`.
- GitHub Actions source-contract workflow.
- `.gitignore` updated so application `Cargo.lock` is committed while build outputs and local caches are excluded.
- Updated two stale host-render tests after cleanup: the enlarged typography sample now fits the mock display, and the Home chrome assertion now reflects the fixed dark footer.

## Source-code policy

No hardware-facing Rust module was removed during cleanup. The dead material identified in the uploaded archive was patch infrastructure, extracted overlay payloads, package fixtures and scratch artifacts. The physically verified firmware modules remain intact.


## v0.16.1 additions

- Reader-owned persistent `STATE.TXT`, `POSITS.TXT`, `RECENT.TXT`, `MARKS.TXT` and `CACHE/<8HEX>.CCH` records.
- Atomic Reader-state `.TMP` / `.BAK` replacement and corrupt-record fallback.
- Persistent Continue Reading, Recent-tab and bookmark-list behavior.


## v0.16.2 additions

- Reader category status copy updated to `READY` for Continue Reading, Library and Bookmarks.
- TXT Unicode punctuation normalization, basic Latin transliteration and simple underscore-emphasis cleanup.
- Reader-owned `PREFS.TXT` persistence with atomic `.TMP` / `.BAK` replacement.
- Reader themes, Portrait / Landscape page orientation, Small / Medium / Large / XLarge font sizes, Inter / Atkinson Hyperlegible / Serif book fonts and Show Progress toggle.
- Generated printable-ASCII DejaVu Serif raster arrays without raw font-file distribution.

## v0.16.4 additions

- Shared Classic / High Contrast Reader body geometry.
- High Contrast border outside the text viewport plus explicit body glyph clipping guards.
- Multiline Project Gutenberg underscore-emphasis cleanup with source-byte-anchor preservation.

## v0.16.7 Reader runtime audit

The Reader persistence audit verifies every generated primary, `.TMP` and `.BAK` filename before write. The runtime uses `POSITS.*` and `<8HEX>.*` only. Bookmark rows now expose a `PAGE` column.

## v0.16.8 Library bookmark-tab presentation alignment

The Library Bookmarks tab now renders saved marks as title plus layout-aware `PAGE N` labels, reports `<n> saved / MARKS.TXT`, and suppresses the unrelated EPUB-placeholder note. The dedicated Bookmarks screen and Reader byte-anchor persistence remain unchanged.
