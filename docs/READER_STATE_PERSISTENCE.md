# Reader state persistence and bookmarks

RustMix Wave v0.16.2 stores bounded Reader-owned records under `/sdcard/RUSTMIX/READER`. The generic File Browser remains read-only.

## Files

```text
/RUSTMIX/READER/STATE.TXT       # most recent TXT position
/RUSTMIX/READER/RECENT.TXT      # bounded recent-book list
/RUSTMIX/READER/MARKS.TXT       # bounded bookmark list
/RUSTMIX/READER/PREFS.TXT       # Reader theme, orientation, font and progress choices
/RUSTMIX/READER/CACHE/*.CCH     # bounded TXT page-anchor cache
```

## Power-safe replacement

Every Reader-owned write is staged into a `.TMP` sibling, synchronized, and renamed into place. The previous primary is retained temporarily as `.BAK`. Startup accepts a valid backup when a primary file is missing or corrupt.

## Cache fingerprint

A TXT cache is accepted only when its fingerprint matches the book path, file size, modified timestamp when available, content format, Reader viewport line count, line width, orientation, book-font family, book-font size and cache-format version. A stale or corrupt cache is ignored; book opening falls back to the saved byte offset or page one.

## Bounds

- Recent books: 16
- Bookmarks: 128
- TXT anchors per cache file: 4096
- Nearby rendered pages in RAM: 8

## Deferred

EPUB state anchors, EPUB TOC records and EPUB cache layouts remain deferred to the reflowable EPUB milestone.

## Reader preferences

`PREFS.TXT` supports:

```text
theme=classic|high-contrast
orientation=portrait|landscape
font_size=small|medium|large|xlarge
book_font=inter|atkinson-hyperlegible|serif
show_progress=true|false
```

Theme and progress changes redraw the page without invalidating pagination. Orientation, book-font and size changes preserve the current source-byte anchor, display the staged loading screen and rebuild the visible page first.

## v0.16.4 per-book positions

`/RUSTMIX/READER/POSITIONS.TXT` retains up to 64 last-read TXT locations. Books and Files reopen through this map; explicit bookmark jumps remain higher authority. The file uses the existing `.TMP` / `.BAK` atomic replacement path.
