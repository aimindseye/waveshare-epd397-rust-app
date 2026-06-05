# EPUB Chapter-Aware Presentation

`v0.17.1` extends the accepted bounded reflowable EPUB reader without changing TXT behavior.

## Reader page labels

EPUB spine extraction now retains readable chapter boundaries. Reader layout rebuilds a bounded in-RAM page-anchor index for every readable chapter and displays `CH n  PAGE x/y`, where `y` is the total page count inside the current chapter for the active Reader layout. TXT labels remain book-relative.

## Bookmark labels

`MARKS.TXT`, `STATE.TXT`, `POSITS.TXT`, and `RECENT.TXT` retain the canonical byte offset. EPUB locations additionally persist optional chapter number, chapter page number and chapter page total values. Seven-field legacy list records remain readable.

## Library titles

Books and Files rows use the OPF metadata title for readable EPUB archives. Title extraction runs on the bounded EPUB worker stack and falls back to the FAT filename when metadata is unavailable. TXT rows continue to use their filename-derived titles.

## Bounds

Chapter page anchors are capped by `READER_EPUB_PAGE_ANCHOR_LIMIT`. EPUB page reads are clipped at chapter boundaries so a displayed page never silently spills into the following chapter.
