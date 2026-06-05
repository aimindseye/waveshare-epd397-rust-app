# Changelog

## v0.17.2 — Reader E-Ink Font Pack

- Preserve the existing Inter, `serif`, and `atkinson-hyperlegible` Reader preference contracts.
- Upgrade the Reader-only Atkinson raster source to Atkinson Hyperlegible Next Medium without renaming its persisted key.
- Add Literata Medium as a new explicit `literata` Reader font option.
- Keep raw TTF, OTF, WOFF, and WOFF2 files out of the repository; embed printable-ASCII generated Rust arrays only.
- Keep selected font in the existing layout cache fingerprint and staged layout-rebuild route.
- Rebuild chapter-aware EPUB page totals after font changes while preserving byte-offset bookmark anchors.
- Keep TXT and EPUB font behavior aligned.

## v0.17.1 — EPUB Watchdog and Memory-Pressure Repair

- Added cooperative 1 ms pauses after every four chapter-page anchors so large EPUB first-page indexing lets the ESP-IDF idle task feed its watchdog.
- Persisted and released the active Reader session before opening another book, preventing the previous flattened EPUB document from overlapping the next parser-worker allocation.
- Moved EPUB documents into layout rebuilds instead of cloning their flattened text buffers.
- Removed the full-document clone previously used for EPUB TOC jumps.
- Reduced metadata-only OPF-title worker stacks from 64 KB to 32 KB while preserving the accepted 64 KB full EPUB parser-worker budget.
- Added runtime markers, source-contract guards and physical smoke-test guidance for watchdog-free repeated EPUB opens.
- Preserved TXT behavior, chapter-aware page labels, chapter-aware bookmarks, OPF Library titles, FAT 8.3 persistence, sleep images and network suspension.

## v0.17.1 — EPUB Chapter-Aware Presentation

- Added readable EPUB chapter boundaries alongside the flattened UTF-8 document.
- Pre-indexed bounded chapter-relative EPUB page anchors for `CH n  PAGE x/y` Reader labels.
- Persisted optional EPUB chapter/page labels in `MARKS.TXT`, while retaining backward compatibility with seven-field TXT and legacy records.
- Rendered EPUB bookmarks as `CH n` plus `P x/y` in Library and Bookmarks screens.
- Resolved OPF metadata titles on the existing bounded EPUB worker stack for Books and Files Library rows, with FAT filename fallback.
- Preserved the accepted TXT Reader, FAT 8.3 state files, bookmark byte-offset authority, preferences, layout rebuild flow and parser stack isolation.

## v0.17.0 — EPUB Parser Stack Isolation Repair

- Moved EPUB archive parsing, DEFLATE expansion and XHTML flattening off the 16 KB firmware main task.
- Added a short-lived `epub-parser` worker with an explicit 64 KB stack and synchronous join boundary.
- Kept staged Reader loading, TXT rendering, FAT 8.3 persistence, bookmarks, preferences, sleep images and network suspension unchanged.
- Added host and repository-contract guards for the worker boundary and stack budget.

## v0.17.0 — Reflowable EPUB Foundation

- Repaired quoted XML attribute scanning after opening element names so valid `container.xml` rootfiles resolve correctly.
- Added EPUB tokenizer and plural-wrapper regression tests.
- Restored validator-required v0.16.4–v0.16.8 physical smoke-test records.
- Removed patch backup residue, added a source backup-artifact contract guard and excluded backup files from release ZIPs.
- Removed unused EPUB imports from the Reader host-test module.
- Activated `.EPUB` and FAT-friendly `.EPU` rows in Reader > Library.
- Added bounded ZIP central-directory parsing with stored and DEFLATE member extraction.
- Added `META-INF/container.xml`, OPF manifest, OPF spine and XHTML text extraction.
- Reflowed EPUB chapter text through the accepted Reader typography, orientation, High Contrast and paragraph-alignment pipeline.
- Added EPUB3 nav, EPUB2 NCX and spine-derived TOC rows under Reader Options > Table of Contents.
- Reused Continue Reading, Recent, bookmarks, per-book resume and byte-offset authority for unchanged EPUB source files.
- Preserved TXT SD-backed FAT 8.3 anchor caches and Power-key network-suspended sleep.

## v0.16.8 — Library Bookmark Tab Rendering Alignment

- Aligned `Reader > Library > Bookmarks` with the dedicated Bookmarks screen.
- Rendered bookmark rows as `PAGE N` labels using layout-aware resolution with stored-page fallback.
- Changed the bookmark-tab status strip to `<n> saved / MARKS.TXT`.
- Kept Books and Files rows on the existing `TXT / OPEN` presentation.
- Removed the unrelated EPUB-placeholder note from the Bookmarks tab only.

## v0.16.7 — Reader FAT 8.3 Runtime Completion and Bookmark Page Labels

- Audited every Reader-owned writable primary, TMP and BAK path.
- Kept per-book positions on `POSITS.*` and cache files on `<8HEX>.*` without a `B` prefix.
- Added layout-aware bookmark `PAGE N` labels while keeping byte offsets authoritative.
- Suppressed repeated identical degraded persistence log messages until status changes.

## v0.16.6 — Reader FAT 8.3 Persistence Filename Repair

- Replaced writable per-book resume files `POSITIONS.TXT`, `POSITIONS.TMP`, and `POSITIONS.BAK` with FAT 8.3-safe `POSITS.TXT`, `POSITS.TMP`, and `POSITS.BAK`.
- Added read-only migration fallback for legacy `POSITIONS.TXT` and `POSITIONS.BAK` when the new short-name-safe primary is absent.
- Removed the extra `B` prefix from TXT anchor-cache basenames so generated cache files use exactly eight hexadecimal characters, such as `ED9B69AF.CCH`.
- Added runtime FAT 8.3 guards for Reader-owned primary, `.TMP`, and `.BAK` write paths.
- Preserved per-book resume semantics, explicit bookmark priority, Reader Preferences navigation, paragraph alignment, first-page-first loading, lazy indexing, and sleep-route restoration.

# Changelog

## v0.16.5 — Reader Preferences Settings-Style Navigation Alignment

- Changed Reading Preferences to the same Settings-style row interaction used elsewhere in the UI.
- UP / DOWN now moves the highlighted preference row.
- SELECT now changes only the highlighted row value and persists it immediately.
- HOLD BOOT returns to Reader Options.
- Layout-sensitive changes still enter the staged current-page rebuild path immediately.
- Redraw-only Theme and Show Progress changes remain in-place updates.
- Preserved per-book resume, bookmark authority, paragraph alignment, High Contrast geometry, TXT cleanup and sleep restoration.

## v0.16.4 — Per-Book Resume and Reader Controls Alignment

- Added bounded per-book resume persistence under `/sdcard/RUSTMIX/READER/POSITIONS.TXT`.
- Resume records are fingerprinted by path, source size, modified timestamp when available, and format.
- Books opened from Continue Reading, Books, and Files resume from their saved per-book positions.
- Explicit bookmark jumps remain higher-authority than general per-book resume positions.
- Split Reader Options actions from the Reading Preferences editor.
- Standardized Reader action-menu controls as UP/DOWN move, SELECT activate, and HOLD BOOT back.
- Standardized Reader preference controls as UP/DOWN change, SELECT next, and HOLD BOOT save/back.
- Added Reader Paragraph Alignment with Justified as the default and Left, Center, and Right alternatives.
- Paragraph alignment persists in `PREFS.TXT` and participates in the TXT layout-cache fingerprint.
- Preserved the v0.16.3 shared Reader viewport, High Contrast geometry repair, multiline Gutenberg emphasis cleanup, persistence, staged loading, lazy caching, and sleep-route restoration.

## v0.16.3 — Reader High-Contrast Layout and TXT Emphasis Cleanup

- Repaired High Contrast Reader geometry by using one shared body-content rectangle for Classic and High Contrast.
- Moved the High Contrast border outside the shared text viewport and added explicit top padding below the status strip.
- Added right-edge and bottom-edge pixel clip guards for Reader body glyphs.
- Kept Reading Theme redraw-only and retained the global ghost-clearing refresh request after a theme change.
- Removed multiline Project Gutenberg `_..._` emphasis markers before wrapping while preserving word-internal underscores and repeated underscore separators.
- Preserved original TXT byte anchors for Continue Reading and bookmarks.

## v0.16.2 — Reader UX Repair and Preferences Foundation

- Updated Reader category rows so Continue Reading, Library and Bookmarks all report `READY`.
- Added TXT punctuation normalization before pagination while preserving source-byte anchors for resume and bookmarks.
- Added smart-quote, dash, ellipsis, non-breaking-space and bounded Latin transliteration fallbacks.
- Removed simple `_text_` emphasis markers from rendered TXT pages.
- Added Reader-owned `/sdcard/RUSTMIX/READER/PREFS.TXT` persistence with atomic `.TMP` / `.BAK` replacement.
- Added Reading Theme, Portrait / Landscape orientation, Small / Medium / Large / XLarge book font sizes, Inter / Atkinson Hyperlegible / Serif book fonts and Show Progress.
- Added generated printable-ASCII DejaVu Serif Reader atlases without distributing raw font files.
- Added first-page-first layout-cache rebuilds after orientation, book-font or book-size changes.

## v0.16.1 — Reader State Persistence and Bookmarks

- Added Reader-owned `/sdcard/RUSTMIX/READER` state files: `STATE.TXT`, `RECENT.TXT` and `MARKS.TXT`.
- Added persistent Continue Reading, Recent-tab history and add/remove bookmark behavior.
- Added bookmark-list navigation from the Reader category, Library Bookmarks tab and Reader options.
- Added SD-backed TXT anchor caches under `/sdcard/RUSTMIX/READER/CACHE` with path, size, modified-time, format and layout fingerprint validation.
- Added `.TMP` / `.BAK` atomic replacement and corrupt-state fallback behavior.
- Preserved first-page-first opening, lazy nearby RAM caching, TXT TOC `NONE`, Clear Ghosting, BOOT Back and sleep-route restoration.

## v0.16.0 — Reader Library and TXT Foundation

- Activated `Reader > Library` with `/sdcard/RUSTMIX/BOOKS` scanning.
- Added TXT detection, UTF-8 / BOM handling and Windows-1252 fallback.
- Added responsive staged opening, first-page-first rendering and lazy nearby-page RAM caching.
- Added Reader page header, progress, options shell, visible TXT TOC `NONE` row and manual ghost-clearing refresh.
- Kept EPUB / EPU rows visible as clean architecture placeholders for v0.17.0.
- Preserved Calendar, Unit Converter, sleep-image mode, wake guard and network suspension.

## v0.15.0 — Unit Converter Foundation

- Added `Tools > Unit Converter`.
- Added offline fixed-point Length, Mass, Temperature and Volume conversions.
- Added adjustable `0.1`, `1`, `10` and `100` step sizes.
- Kept Calendar, sleep-image mode, Power-key wake guard and network suspension unchanged.
- Synchronized stale host-render tests with the enlarged typography atlas and fixed dark Home footer.

## v0.14.1 — Power-Key Sleep Entry Wake-Guard Repair

- Added a `900 ms` quiet-window guard after sleep-image entry.
- Suppressed queued AXP2101 PEK events caused by the entry press.
- Preserved deliberate second-press wake and RTC-alarm wake behavior.

## v0.14.0 — Calendar Foundation

- Added `Productivity > Calendar`.
- Added a read-only RTC-localized monthly view with Day and Month navigation modes.
- Added Gregorian calendar host tests for the supported `2000..=2099` RTC range.

## v0.13.5 — Home Dashboard Redesign

- Added simplified dark Home header, localized date/time row, summary strip, cleaner cards and fixed footer.

## v0.13.4 — Weather Fetch Resilience

- Added bounded non-blocking retries with `2`, `5`, and `15` second backoff.
- Added retry classification for TLS, transport and selected HTTP status failures.
- Added last-known-good in-memory cache retention with `RETRYING` and `STALE` UI states.

## v0.13.3 — Secondary Screen Readability Reflow

- Split dense diagnostics into readable overview and detail pages.
- Added Device Info pagination.
- Removed synthetic Settings Back rows where BOOT long-press Back is available.

## v0.13.2 — Typography Scale Increase

- Regenerated Inter and Atkinson Hyperlegible 1-bpp atlases at larger raster sizes.
- Preserved Compact, Standard and Large profiles.

## v0.13.1 — Global UI Typography Foundation

- Added persistent Display preferences in `/sdcard/RUSTMIX/DISPLAY.TXT`.
- Added GPIO0 BOOT long-press hierarchical Back.

## v0.13.0 — Main Category Navigation Shell

- Added Reader, Productivity, Games, Tools and Settings categories.

## v0.12.3 — Random Sleep-Image Selector

- Added hardware-random BMP selection with immediate-repeat avoidance.

## v0.12.2 — Network-Suspended Sleep Images

- Added SNTP, Wi-Fi and Weather suspension while a static sleep image is visible.

## 0.16.7 — Reader FAT 8.3 Runtime Completion and Bookmark Page Labels

- Audited every Reader-owned writable primary, TMP and BAK path.
- Kept per-book positions on `POSITS.*` and cache files on `<8HEX>.*` without a `B` prefix.
- Added layout-aware bookmark `PAGE N` labels while keeping byte offsets authoritative.
- Suppressed repeated identical degraded persistence log messages until status changes.
