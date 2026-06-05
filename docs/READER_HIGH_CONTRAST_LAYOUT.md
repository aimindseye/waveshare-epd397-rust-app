# Reader High Contrast layout boundary

RustMix Wave v0.16.3 keeps pagination stable when Reading Theme changes.

## Shared body rectangle

Classic and High Contrast use the same body-content rectangle, wrapping width and line count. High Contrast draws a stronger border outside the body viewport. Reader body text begins below an explicit top padding boundary and uses right / bottom pixel clip guards as a final safety net for proportional glyphs.

## Redraw-only theme changes

Changing Reading Theme persists `PREFS.TXT`, redraws the current page and requests a global ghost-clearing refresh. It does not invalidate the layout-aware TXT anchor cache.

## Multiline underscore emphasis cleanup

Before wrapping, TXT normalization removes Project Gutenberg emphasis delimiters even when the opening and closing underscores occur on different source lines. Word-internal underscores (`file_name`) and repeated separators (`_____`) remain visible. Original source-byte anchors remain authoritative for Continue Reading and bookmarks.
