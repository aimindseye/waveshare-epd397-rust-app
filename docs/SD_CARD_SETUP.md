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
