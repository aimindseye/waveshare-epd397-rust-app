# Reader UX repair and preferences

RustMix Wave v0.16.2 completes the TXT Reader UX repair before reflowable EPUB work begins.

## Reader category status

`Continue Reading`, `Library`, and `Bookmarks` are active `READY` routes. The Reader landing page no longer displays persistence-shell copy.

## TXT normalization

TXT decoding remains byte-aware (`UTF-8`, UTF-8 BOM, or Windows-1252 fallback). After decoding and before wrapping, the Reader normalizes unsupported punctuation into the printable-ASCII bitmap boundary. Smart quotes, apostrophes, dashes, ellipses, non-breaking spaces, common accented Latin characters, and simple `_emphasis_` markers are handled while persisted bookmarks continue to use original source-byte anchors.

## Reader-only preferences

Preferences are independent from global `/RUSTMIX/DISPLAY.TXT` UI typography and persist under:

```text
/RUSTMIX/READER/PREFS.TXT
```

Supported values:

```text
version=1
theme=classic | high-contrast
orientation=portrait | landscape
font_size=small | medium | large | xlarge
book_font=inter | atkinson-hyperlegible | serif | literata
paragraph_alignment=justified | left | center | right
show_progress=true | false
```

`PREFS.TXT` uses the existing Reader `.TMP` / `.BAK` atomic-replacement path and corrupt-record fallback.

## Cache invalidation

Orientation, font size, and book font affect pagination and are included in TXT anchor-cache fingerprints. Changing one of those values preserves the current source-byte anchor, returns immediately to the staged loading screen, rebuilds the visible page first, and continues nearby indexing lazily.

Reading theme and progress visibility redraw the current page but do not invalidate page anchors.

## Reader e-ink font pack

The existing Serif option continues to use generated printable-ASCII DejaVu Serif raster arrays. The persisted `serif` key is unchanged. The persisted `atkinson-hyperlegible` key is also unchanged while its Reader-only raster source is upgraded to Atkinson Hyperlegible Next Medium. Literata is a new explicit option backed by generated Literata Medium strikes. Inter continues to reuse the UI-backed Reader strikes. Raw font files are not distributed. See `docs/READER_EINK_FONT_PACK.md`.

## v0.16.4 theme geometry rule

Reading Theme remains a redraw-only preference. Classic and High Contrast share the same text rectangle, wrapping width, line count and cache-fingerprint inputs. High Contrast adds chrome outside the viewport and triggers a global ghost-clearing refresh after selection.

## v0.16.4 paragraph alignment and controls

Reader Options now contains action rows only. Reading Preferences is a separate editor: UP/DOWN changes the active value, SELECT advances to the next preference, and HOLD BOOT saves and returns. Paragraph Alignment supports `justified` (default), `left`, `center`, and `right`. Paragraph alignment participates in the TXT cache fingerprint so resumed layout remains deterministic.
