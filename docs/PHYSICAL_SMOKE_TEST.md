# RustMix Wave v0.16.4 consolidated physical smoke test

1. Validate, host-test, build and flash the firmware.
2. Confirm `version=0.16.8 milestone=library-bookmark-tab-rendering-alignment`.
3. Copy at least one TXT book into `/RUSTMIX/BOOKS`.
4. Open `Reader > Library`, open the TXT file and confirm the staged loading screen appears before page one.
5. Advance at least two pages and return to Home. Reboot the device.
6. Open `Reader > Continue Reading` and confirm the saved page restores.
7. Open `Reader > Library`, cycle to `Recent`, and confirm the saved book is listed.
8. Open the TXT page, select Reader Options, add a bookmark and confirm the page status becomes `MARKED`.
9. Open Reader Options > Bookmarks and confirm the saved mark opens.
10. Remove the same bookmark and confirm the bookmark list becomes empty.
11. Confirm `/RUSTMIX/READER/STATE.TXT`, `POSITS.TXT`, `RECENT.TXT`, `MARKS.TXT` and `CACHE/<8HEX>.CCH` exist.
12. Temporarily rename a primary Reader state file to its `.BAK` sibling and confirm startup remains non-fatal.
13. Confirm TOC remains visible and reports `NONE` for TXT.
14. Confirm Clear Ghosting performs a global refresh.
15. Enter Power-key sleep from a TXT page, wait for the wake guard, wake again and confirm the Reader page restores.
16. Confirm Calendar, Unit Converter, alarms, random sleep images and network restoration remain intact.


## Reader UX preferences

Run `docs/V0.16.4-PHYSICAL-SMOKE-TEST.md` after flashing v0.16.4.

## v0.16.7 Reader FAT 8.3 runtime and bookmark labels

- Turn pages and confirm no `os error 22` Reader cache messages.
- Confirm `POSITS.TXT` and `<8HEX>.CCH` files appear on the SD card.
- Add two marks in one book and confirm distinct `PAGE` values in Bookmarks.

## v0.16.8 Library bookmark-tab rendering

- Add two bookmarks in the same TXT book on different pages.
- Open `Reader > Library > Bookmarks` and confirm the status strip reports `<n> saved / MARKS.TXT`.
- Confirm each row renders `PAGE N`, including distinct numbers for multiple marks from the same book.
- Select each Library bookmark row and confirm it opens the explicit saved passage.
- Cycle back to Books and Files and confirm rows still render `TXT / OPEN`.
- Confirm the EPUB-placeholder note is absent on Bookmarks but remains present on Books and Files.
