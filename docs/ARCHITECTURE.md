# Architecture

RustMix Wave is intentionally split into hardware-independent product logic and ESP-IDF-specific runtime wiring.

## Design rules

1. Keep `src/main.rs` focused on peripheral ownership and event-loop orchestration.
2. Keep pure application state, routing, conversion logic, calendar math and parser behavior host-testable.
3. Treat the SD card as optional. Missing storage must not block panel boot.
4. Keep File Browser operations read-only and root confined.
5. Keep power transitions explicit: render first, then change the e-paper rail or network runtime.
6. Avoid putting secrets in serial logs. Wi-Fi passwords are never logged.

## Product UI

```text
src/app/
├── menu.rs             # Data-driven Home and category entries
├── router.rs           # Hierarchical routes and parent relationships
├── state.rs            # Hardware-independent application state
├── display.rs          # DISPLAY.TXT preferences
├── typography/         # Generated 1-bpp font atlases and semantic roles
├── widgets/            # Header, footer, status row, cards and dashboard components
└── screens/            # Individual product screens
```

The active Home structure is:

```text
Reader
Productivity
Games
Tools
Settings
```

GPIO0 BOOT long-press performs hierarchy-level Back navigation. Category lists do not render synthetic Back rows.

## Runtime ownership

```text
src/main.rs
├── panel and framebuffer
├── SDMMC mount
├── shared I2C bus
├── AXP2101 PMIC
├── PCF85063 RTC and GPIO45 alarm monitor
├── SHTC3 environment sensor
├── QMI8658 IMU
├── ES8311 audio runtime
├── Wi-Fi and SNTP runtime
├── Weather fetch scheduling
└── product event loop
```

## Sleep-image mode

The MCU remains active in sleep-image mode. The static panel image and paused network services reduce active peripheral work while preserving reliable PMIC and RTC-alarm polling.

```text
Power short press
→ stop audio
→ scan /sdcard/RUSTMIX/SLEEP
→ choose hardware-random anti-repeat BMP
→ suspend SNTP / Wi-Fi / Weather
→ global panel refresh
→ panel deep sleep and ALDO3 off
→ start 900 ms wake guard
```

A deliberate second Power short press after the quiet window restores the previous route and network services.

## Weather

Weather logic is split between:

```text
src/weather_config.rs    # WEATHER.TXT parsing
src/weather.rs           # Open-Meteo types, parser, retry policy and RAM cache
src/main.rs              # Non-blocking retry scheduler and ESP HTTPS orchestration
```

Retry waits are event-loop scheduled rather than implemented as blocking delays.

## Offline applications

### Calendar

`src/calendar.rs` contains Gregorian calendar math and session state. `src/app/screens/calendar.rs` renders the read-only localized monthly view.

### Unit Converter

`src/unit_converter.rs` contains fixed-point conversion rules and bounds. `src/app/screens/unit_converter.rs` renders the interactive offline screen. It performs no network, filesystem or peripheral access.

## Storage boundary

`src/storage.rs` owns a bounded, read-only browser facade:

- optional `/sdcard` mount
- directory-first sorting
- FAT metadata fallback classification
- retry-limited scans
- bounded text previews
- root-confined navigation

Display preferences are a narrow exception: `src/app/display.rs` reads and writes only `/sdcard/RUSTMIX/DISPLAY.TXT`.
