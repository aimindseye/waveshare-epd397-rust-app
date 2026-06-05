# Reader Library and TXT Foundation

RustMix Wave v0.16.2 retains the TXT Reader foundation and persistent Reader state, then adds TXT normalization and Reader-only preferences.

## Supported now

- Reader landing routes: Continue Reading, Library and Bookmarks.
- `/sdcard/RUSTMIX/BOOKS` scan.
- `.TXT` rendering.
- UTF-8, UTF-8 BOM and Windows-1252 fallback decoding.
- Coarse stage-based opening screen.
- First-page-first display.
- Lazy byte-anchor indexing and bounded nearby-page RAM cache.
- Reader header, page label and cache progress.
- Reader options with visible Table of Contents row (`NONE` for ordinary TXT).
- Manual Clear Ghosting global refresh.
- BOOT long-press Back and Power-key sleep route restoration.

## EPUB architecture boundary

`.EPUB` and short-name-safe `.EPU` files now open through the bounded v0.17.0 reflowable EPUB foundation. TXT keeps its existing byte-aware decoder and SD-backed anchor cache. EPUB uses OPF-spine extraction, RAM-bounded text reflow and a real TOC route without replacing the TXT Reader session model.

## Persistent Reader state

- Continue Reading and per-book resume.
- Recent-books persistence.
- Bookmark add / remove persistence with page labels.
- SD-backed TXT page-anchor cache.
- Atomic Reader state files with FAT 8.3-safe names.

## SD-card layout

```text
/RUSTMIX/BOOKS/*.TXT
/RUSTMIX/BOOKS/*.EPUB
/RUSTMIX/BOOKS/*.EPU
/RUSTMIX/READER/STATE.TXT
/RUSTMIX/READER/POSITS.TXT
/RUSTMIX/READER/RECENT.TXT
/RUSTMIX/READER/MARKS.TXT
/RUSTMIX/READER/PREFS.TXT
/RUSTMIX/READER/CACHE/<8HEX>.CCH
```


See [`READER_STATE_PERSISTENCE.md`](READER_STATE_PERSISTENCE.md) for the v0.16.1 state-file and cache contracts.

## TXT normalization

Before wrapping, the Reader normalizes common Unicode punctuation and bounded Latin characters into printable ASCII while retaining each source character's original next-byte offset. This keeps saved positions and bookmarks stable while avoiding `?` noise from glyphs outside the embedded atlas. Simple `_text_` emphasis markers are removed from the visible page.

## Reader typography and layout

Reader pages use independent preferences from the operating-system UI. Inter and Atkinson Hyperlegible reuse existing generated strikes. Serif uses generated printable-ASCII DejaVu Serif strikes in Small, Medium, Large and XLarge sizes. Raw font files are not distributed. Portrait and Landscape page geometry have separate bounded line and column counts and therefore separate anchor-cache fingerprints.

## v0.16.4 High Contrast geometry and multiline emphasis

Classic and High Contrast share one text viewport and the same pagination input. High Contrast draws its heavier border outside that rectangle. Reader body rendering applies explicit right and bottom clip guards, while top padding keeps the first text line below the status strip. Theme changes request a global ghost-clearing refresh but do not invalidate TXT anchors.

TXT normalization now removes multiline Project Gutenberg emphasis delimiters such as `_first line ... final line_` before wrapping. Word-internal underscores such as `file_name` and repeated separators such as `_____` remain intact. Saved positions continue to use original source-byte anchors.


## v0.17.0 follow-on

The deferred EPUB boundary is now active in `src/epub.rs`. TXT pagination and TXT SD-backed anchor caches remain unchanged.
