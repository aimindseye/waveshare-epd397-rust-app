# SD-card setup

Use a FAT-formatted SD card. The firmware mounts it at `/sdcard` and expects the following removable-storage layout:

```text
/RUSTMIX/
├── WIFI.TXT
├── WEATHER.TXT
├── ALARMS.TXT
├── DISPLAY.TXT
└── SLEEP/
    ├── SLEEP.BMP
    └── *.BMP
```

## Install example files

```bash
./scripts/install-sd-examples.sh /Volumes/YOUR_SD_CARD
```

Existing files are preserved by default. To overwrite existing example paths intentionally:

```bash
./scripts/install-sd-examples.sh --force /Volumes/YOUR_SD_CARD
```

## Wi-Fi

Copy `examples/sd-card/RUSTMIX/WIFI.TXT.example` to `/RUSTMIX/WIFI.TXT` and edit locally:

```text
ssid=YOUR_NETWORK
password=YOUR_PASSWORD
timezone=America/New_York
ntp_server=pool.ntp.org
```

Do not commit a real password.

## Weather

Copy `examples/sd-card/RUSTMIX/WEATHER.TXT.example` to `/RUSTMIX/WEATHER.TXT`:

```text
provider=open-meteo
location=New York, NY
latitude=40.7128
longitude=-74.0060
timezone=America/New_York
refresh_minutes=30
```

Weather is optional. Missing or failing Weather configuration does not block panel boot.

## Alarms

Copy `examples/sd-card/RUSTMIX/ALARMS.TXT.example` to `/RUSTMIX/ALARMS.TXT`:

```text
snooze_minutes=10
alarm=Workday,07:30,weekdays,on,recurring
alarm=Weekend,09:00,weekends,off,recurring
alarm=Appointment,16:45,2026-06-10,on,once
```

The UI can edit schedules during the active session. Persistent alarm edits remain SD-file based.

## Display

Copy `examples/sd-card/RUSTMIX/DISPLAY.TXT.example` to `/RUSTMIX/DISPLAY.TXT`:

```text
font_family=inter
font_size=standard
```

Supported values:

```text
font_family=inter|atkinson-hyperlegible
font_size=compact|standard|large
```

Display changes are saved back to this file by the Settings screen.

## Sleep images

Native sleep images must be:

```text
800 × 480
1-bpp monochrome Windows BMP
uncompressed
```

Install the bundled samples only:

```bash
./scripts/install-sleep-images.sh /Volumes/YOUR_SD_CARD
```


## Reader books

Create `/RUSTMIX/BOOKS` and copy TXT or ordinary reflowable EPUB books into it. v0.17.0 renders `.TXT`, `.EPUB`, and FAT-friendly `.EPU` books. EPUB CSS layout, images, DRM and fixed-layout packages remain deferred.


## Reader-owned state

The firmware creates `/RUSTMIX/READER` and `/RUSTMIX/READER/CACHE` automatically after the first TXT page opens. Do not hand-edit active Reader state while the device is running.

```text
/RUSTMIX/READER/STATE.TXT
/RUSTMIX/READER/RECENT.TXT
/RUSTMIX/READER/MARKS.TXT
/RUSTMIX/READER/PREFS.TXT
/RUSTMIX/READER/CACHE/<8HEX>.CCH
```

Reader writes use temporary and backup siblings (`.TMP`, `.BAK`) so an interrupted update can recover on the next boot.

## Reader preferences

`PREFS.TXT` is created automatically after the first Reader preference change. Supported values:

```text
version=1
theme=classic|high-contrast
orientation=portrait|landscape
font_size=small|medium|large|xlarge
book_font=inter|atkinson-hyperlegible|serif
show_progress=true|false
```

## Reader per-book resume state

The Reader creates `/RUSTMIX/READER/POSITS.TXT` automatically when books are opened. It stores a bounded map of last-read source-byte anchors for previously opened books.

## FAT 8.3-safe Reader persistence filenames

The Reader writes per-book progress to `/RUSTMIX/READER/POSITS.TXT`. Cache files use exactly eight hexadecimal basename characters such as `/RUSTMIX/READER/CACHE/ED9B69AF.CCH`. Temporary and backup siblings use `.TMP` and `.BAK`. Legacy `POSITIONS.TXT` is read-only migration input when the short-name-safe primary is absent.

## Reader runtime files after v0.16.7

```text
/RUSTMIX/READER/POSITS.TXT
/RUSTMIX/READER/CACHE/<8HEX>.CCH
```

Generated Reader cache basenames are exactly eight hexadecimal characters and do not use a leading prefix.
