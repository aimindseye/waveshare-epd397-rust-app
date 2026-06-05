# EPUB Watchdog and Memory-Pressure Repair

## Scope

Large EPUB books already open correctly after parser stack isolation, but v0.17.1 chapter-aware page totals initially indexed every chapter page synchronously without yielding. On ESP32-S3 this could keep CPU0 busy long enough for ESP-IDF to report `IDLE0` task-watchdog warnings even though the Reader eventually opened the page.

Repeated EPUB switches could also retain the previous session's flattened text and chapter anchors while starting the next 64 KB parser worker. Under memory pressure, `pthread` could report `Failed to create task!` and the parser worker returned `Not enough space (os error 12)`.

## Repair

- The chapter-page indexer pauses for 1 ms after each four generated anchors. This bounded cooperative pause lets the idle task feed its watchdog while preserving deterministic chapter totals.
- Explicit book opens persist and release the previous active session before parsing the next book.
- Layout rebuilds move the existing EPUB document into the loading state instead of cloning it.
- TOC jumps read the requested page through a borrowed document rather than cloning the full EPUB document.
- Metadata-only OPF title workers use a separate 32 KB stack. Full ZIP, DEFLATE and XHTML parsing remains on the accepted 64 KB `epub-parser` worker.

## Preserved contracts

TXT Reader behavior, EPUB parser-stack isolation, chapter-relative `CH n PAGE x/y` labels, bookmark byte-offset authority, chapter-aware bookmark labels, OPF Library titles, FAT 8.3 Reader persistence, Reader Preferences, High Contrast geometry, network suspension and sleep images remain unchanged.
