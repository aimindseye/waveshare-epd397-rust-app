# Changelog

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
