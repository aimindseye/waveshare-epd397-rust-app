# Reader per-book resume and controls alignment

RustMix Wave v0.16.4 adds `/RUSTMIX/READER/POSITS.TXT`, a bounded 64-record last-position map. Opening a TXT book from Books or Files restores that book's validated saved anchor. Continue Reading remains the global shortcut and bookmark jumps remain explicit higher-authority positions.

Reader Options now exposes actions only. Reading Preferences is a separate Settings-style editor with UP/DOWN move, SELECT change, and HOLD BOOT back behavior. Each SELECT change persists immediately.

Paragraph Alignment options are `justified` (default), `left`, `center`, and `right`. The alignment is persisted in `PREFS.TXT` and included in TXT cache fingerprints.


v0.16.6 stores future writes under FAT 8.3-safe `POSITS.TXT` and accepts legacy `POSITIONS.TXT` read-only for migration.

## Bookmark page labels

Bookmark byte offsets remain authoritative. The Bookmarks screen shows a `PAGE` column resolved from the active layout when nearby anchors are available and otherwise falls back to the stored page index.
