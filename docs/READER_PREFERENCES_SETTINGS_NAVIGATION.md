# Reader Preferences Settings-style navigation

RustMix Wave v0.16.5 aligns Reading Preferences with the Settings-style row editor used elsewhere in the firmware.

Controls:

```text
UP / DOWN
→ move between preference rows

SELECT
→ change the highlighted preference value

HOLD BOOT
→ return to Reader Options
```

Preference changes persist immediately to `/RUSTMIX/READER/PREFS.TXT`.

Redraw-only settings:

```text
Reading Theme
Show Progress
```

Layout-sensitive settings:

```text
Orientation
Book Font Size
Book Font
Paragraph Alignment
```

Layout-sensitive changes preserve the original TXT byte anchor, enter the responsive `Updating layout cache` screen, rebuild the current page first, and continue nearby indexing lazily.
