# Library Bookmark Tab Rendering Alignment

Version: v0.16.8

`Reader > Library > Bookmarks` now uses the same page-label presentation contract as the dedicated Bookmarks screen. Each Library bookmark row renders the book title and a layout-aware `PAGE N` label. The persisted TXT byte offset remains the authoritative jump anchor; the stored page index remains the fallback label when the active layout cache cannot resolve a newer label.

The bookmark-tab status strip reports `<n> saved / MARKS.TXT`. Multiple marks within the same book remain separate rows. The EPUB-placeholder note is intentionally hidden on the Bookmarks tab while Books and Files keep their existing `TXT / OPEN` presentation and EPUB-placeholder note.
