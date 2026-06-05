# Reflowable EPUB Foundation

RustMix Wave v0.17.0 activates bounded reflowable EPUB opening without replacing the accepted TXT Reader or Reader-owned persistence contracts.

## Supported EPUB slice

- `.EPUB` and FAT-friendly `.EPU` library rows open from Books and Files.
- ZIP central-directory parsing is bounded.
- Stored and DEFLATE members are supported through `miniz_oxide`.
- `META-INF/container.xml` selects one OPF package.
- OPF manifest and spine records select XHTML reading-order documents.
- XHTML text is flattened into a bounded UTF-8 RAM buffer and reflowed through the existing Reader typography, paragraph alignment and orientation pipeline.
- EPUB3 navigation documents and EPUB2 NCX files populate `Reader Options > Table of Contents`.
- When a package has no explicit navigation document, a bounded spine-derived TOC is used.
- TOC selections jump by flattened-text byte offset.
- Continue Reading, Recent, bookmarks and per-book resume reuse the existing Reader state files.

## Bounds

- Archive: 16 MiB
- ZIP entries: 512
- One compressed member: 2 MiB
- One expanded member: 4 MiB
- Flattened reflow text: 2 MiB
- Manifest items: 256
- Spine items: 128
- TOC rows: 128

## Deferred

CSS layout, images, SVG, media overlays, fixed-layout EPUB, DRM, ZIP64, hyperlinks, footnotes and SD-backed EPUB anchor caches remain deferred. TXT SD-backed anchor caches remain unchanged.
