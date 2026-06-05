# Reader E-Ink Font Pack

RustMix Wave v0.17.2 expands the Reader-only body-font cycle while preserving the accepted TXT and EPUB paths.

## Reader font cycle

```text
Inter
Atkinson
Serif
Literata
```

The stored preference values are:

```text
inter
atkinson-hyperlegible
serif
literata
```

The existing `serif` and `atkinson-hyperlegible` values remain readable and writable without migration. `literata` is the only new stored value.

## Raster sources

- Inter continues to reuse the existing UI-backed Reader strike.
- Atkinson now resolves to Reader-only printable-ASCII raster strikes derived from Atkinson Hyperlegible Next Medium. The global UI Atkinson profile is not changed.
- Serif continues to resolve to the existing Reader-only DejaVu Serif raster strikes.
- Literata resolves to new Reader-only printable-ASCII raster strikes derived from Literata Medium.

Raw font files are input tooling only and are intentionally not copied into the repository or release ZIP. The repository stores generated Rust arrays in:

```text
src/app/reader_atkinson_next_assets.rs
src/app/reader_literata_assets.rs
```

## Pagination and bookmark compatibility

Font selection continues to participate in the Reader layout cache fingerprint. Changing the font triggers the existing staged layout rebuild. TXT pages are rebuilt through the accepted lazy nearby-page cache. EPUB chapter-relative anchors are rebuilt through the accepted cooperative chapter indexer, so `CH n PAGE x/y` totals reflect the selected font.

Bookmarks continue to persist byte offsets as authoritative jump anchors. Layout-aware page and chapter labels are derived presentation metadata.

## Local asset generation

Run the packaged bootstrap script once to install the two source faces and the host-only Python raster tooling. The apply script generates printable-ASCII Rust arrays locally and never copies raw font binaries into the repository.
