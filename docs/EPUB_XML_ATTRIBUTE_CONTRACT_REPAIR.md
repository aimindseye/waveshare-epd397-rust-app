# EPUB XML attribute contract repair

Marker: `rustmix-wave=epub-xml-attribute-tokenizer-repair-ready`

## Root cause

The bounded XML helper correctly distinguishes `<rootfile>` from the plural `<rootfiles>` wrapper, but the reusable attribute tokenizer previously began scanning at the opening element name. When it encountered `rootfile` without an equals sign, recovery advanced across the immediately following `full-path='...'` token. A valid EPUB container therefore failed with `EPUB container rootfile missing`.

## Repair

The tokenizer now skips the opening element name first, then scans bounded quoted attribute tokens. The parser retains namespace-local matching, single-quote and double-quote support, malformed-token recovery, the existing no-DOM architecture and the archive-size limits.

## Regression coverage

Host tests cover extraction of multiple quoted attributes following an element name and rootfile extraction from a container that includes both `<rootfiles>` and `<rootfile full-path='OEBPS/book.opf'/>`.

## Repository contract cleanup

The clean source ZIP now restores the validator-required v0.16.4 through v0.16.8 physical smoke-test checklists. It also removes the stale parser overlay backup and prevents `.bak`, `.orig` and `.rej` files from entering future release archives.

## Preserved Reader baseline

TXT rendering and normalization, `POSITS.TXT`, eight-character cache filenames, bookmark byte-offset authority, Library bookmark page labels, the dedicated Bookmarks screen, Reader Preferences, paragraph alignment, High Contrast layout, sleep images and network suspension remain unchanged.
