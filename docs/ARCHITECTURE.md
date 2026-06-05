# Architecture

RustMix Wave is intentionally split into hardware-independent product logic and ESP-IDF-specific runtime wiring.

## Design rules

1. Keep `src/main.rs` focused on peripheral ownership and event-loop orchestration.
2. Keep pure application state, routing, conversion logic, calendar math and parser behavior host-testable.
3. Treat the SD card as optional. Missing storage must not block panel boot.
4. Keep File Browser operations read-only and root confined.
5. Keep power transitions explicit: render first, then change the e-paper rail or network runtime.
6. Avoid putting secrets in serial logs. Wi-Fi passwords are never logged.

## Product UI

```text
src/app/
├── menu.rs             # Data-driven Home and category entries
├── router.rs           # Hierarchical routes and parent relationships
├── state.rs            # Hardware-independent application state
├── display.rs          # DISPLAY.TXT preferences
├── typography/         # Generated 1-bpp font atlases and semantic roles
├── widgets/            # Header, footer, status row, cards and dashboard components
└── screens/            # Individual product screens
```

The active Home structure is:

```text
Reader
Productivity
Games
Tools
Settings
```

GPIO0 BOOT long-press performs hierarchy-level Back navigation. Category lists do not render synthetic Back rows.

## Runtime ownership

```text
src/main.rs
├── panel and framebuffer
├── SDMMC mount
├── shared I2C bus
├── AXP2101 PMIC
├── PCF85063 RTC and GPIO45 alarm monitor
├── SHTC3 environment sensor
├── QMI8658 IMU
├── ES8311 audio runtime
├── Wi-Fi and SNTP runtime
├── Weather fetch scheduling
└── product event loop
```

## Sleep-image mode

The MCU remains active in sleep-image mode. The static panel image and paused network services reduce active peripheral work while preserving reliable PMIC and RTC-alarm polling.

```text
Power short press
→ stop audio
→ scan /sdcard/RUSTMIX/SLEEP
→ choose hardware-random anti-repeat BMP
→ suspend SNTP / Wi-Fi / Weather
→ global panel refresh
→ panel deep sleep and ALDO3 off
→ start 900 ms wake guard
```

A deliberate second Power short press after the quiet window restores the previous route and network services.

## Weather

Weather logic is split between:

```text
src/weather_config.rs    # WEATHER.TXT parsing
src/weather.rs           # Open-Meteo types, parser, retry policy and RAM cache
src/main.rs              # Non-blocking retry scheduler and ESP HTTPS orchestration
```

Retry waits are event-loop scheduled rather than implemented as blocking delays.

## Offline applications

### Calendar

`src/calendar.rs` contains Gregorian calendar math and session state. `src/app/screens/calendar.rs` renders the read-only localized monthly view.

### Unit Converter

`src/unit_converter.rs` contains fixed-point conversion rules and bounds. `src/app/screens/unit_converter.rs` renders the interactive offline screen. It performs no network, filesystem or peripheral access.

## Storage boundary

`src/storage.rs` owns a bounded, read-only browser facade:

- optional `/sdcard` mount
- directory-first sorting
- FAT metadata fallback classification
- retry-limited scans
- bounded text previews
- root-confined navigation

Display preferences are a narrow exception: `src/app/display.rs` reads and writes only `/sdcard/RUSTMIX/DISPLAY.TXT`. Reader state is a second narrow exception: `src/reader.rs` writes only bounded files below `/sdcard/RUSTMIX/READER`.


### Reader TXT / EPUB persistence and bookmarks

`src/reader.rs` owns the bounded shared Reader domain: `/sdcard/RUSTMIX/BOOKS` scanning, TXT / EPUB classification, staged first-page-first opening, lazy byte-anchor pagination, nearby-page RAM cache, persistent Continue Reading, Recent and bookmarks. Reader-owned state lives below `/sdcard/RUSTMIX/READER`; `.TMP` and `.BAK` siblings provide interrupted-write recovery, while `CACHE/<8HEX>.CCH` retains bounded TXT anchors with fingerprint validation.

`src/epub.rs` is an isolated EPUB boundary. It parses a bounded ZIP central directory, extracts stored or DEFLATE members, resolves `META-INF/container.xml`, parses the OPF manifest and spine, flattens XHTML into a bounded UTF-8 reflow buffer and produces EPUB3-nav, EPUB2-NCX or spine-derived TOC rows. `src/app/screens/reader.rs` renders the shared TXT / EPUB reading page and a real EPUB TOC list while preserving the TXT TOC `NONE` screen.


## Reader preference and typography boundary

The Reader owns `PREFS.TXT` separately from global `DISPLAY.TXT`. Page-layout fingerprints include orientation, book-font family and book-font size. Reader-only serif raster arrays live in `src/app/reader_serif_assets.rs`; raw font binaries are intentionally absent. TXT normalization occurs after byte-aware decoding and before wrapping, so bookmarks continue to use original source-byte anchors.

## v0.16.4 Reader body geometry boundary

Classic and High Contrast use the same pagination rectangle. The High Contrast frame is rendered outside that rectangle, and `Text::draw_clipped` applies a final half-open pixel guard on the Reader body surface. Theme changes therefore redraw the current page without changing TXT page anchors or cache fingerprints. TXT normalization removes bounded multiline Project Gutenberg underscore emphasis delimiters while retaining source-byte anchors, filename-style word-internal underscores and repeated underscore separators.

## v0.16.4 Reader resume and controls boundary

`POSITS.TXT` is the FAT 8.3-safe bounded 64-record per-book resume map separate from global `STATE.TXT`; legacy `POSITIONS.TXT` is accepted read-only for migration. Reader anchor caches use exactly eight hexadecimal basename characters plus `.CCH`, `.TMP`, or `.BAK`. Reader Options is action-oriented; Reading Preferences uses Settings-style UP/DOWN row movement and SELECT value changes. Paragraph alignment is Reader-owned and cache-fingerprinted.

## Reader FAT 8.3 runtime completion and bookmark labels

Reader-owned runtime writes are centralized through the FAT 8.3 guard. Writable per-book positions use `POSITS.*`; TXT anchor caches use `<8HEX>.*`. `POSITIONS.TXT` is legacy read-only migration input. Bookmark labels are resolved from current-layout anchors when available and otherwise fall back to the stored page index.

## Library bookmark-tab presentation boundary

`src/app/screens/reader.rs` uses a bookmark-specific Library-tab presentation branch. `Reader > Library > Bookmarks` resolves each row through the same layout-aware bookmark page-label helper used by the dedicated Bookmarks screen, while the persisted byte offset remains the canonical jump anchor. The Bookmarks tab reports `<n> saved / MARKS.TXT`, omits the EPUB-placeholder note, and keeps duplicate titles as separate marks. Books and Files retain the generic format / open columns.
