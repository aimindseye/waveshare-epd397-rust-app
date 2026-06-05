# Reader FAT 8.3 Runtime Completion and Bookmark Page Labels

Version: v0.16.7

## Runtime filename contract

Writable Reader position files use `POSITS.TXT`, `POSITS.TMP` and `POSITS.BAK`. Writable TXT anchor caches use `<8HEX>.CCH`, `<8HEX>.TMP` and `<8HEX>.BAK`. `POSITIONS.TXT` is read-only legacy migration input.

## Bookmark labels

Bookmarks preserve original TXT byte offsets as canonical anchors. The UI displays a `PAGE` column. When the open session has enough current-layout offsets, the label is resolved for that layout. Otherwise the stored page index is displayed.

## Logging

Repeated identical degraded persistence messages are suppressed until the status changes, reducing serial noise without hiding the first failure.
