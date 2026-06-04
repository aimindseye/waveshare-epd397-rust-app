# RustMix Wave for Waveshare ESP32-S3 e-Paper 3.97

RustMix Wave is a modular Rust / ESP-IDF firmware project for the Waveshare ESP32-S3 3.97-inch e-paper board. The native panel is `800 × 480`; the product UI renders on a logical `480 × 800` portrait canvas.

The current firmware release is **v0.15.0** (`unit-converter-foundation`). It is a hardware-tested product-shell baseline rather than a one-file board demo.

## Current functionality

### Product shell

- Simplified dark Home header.
- Localized date and time row.
- Weather, battery and Wi-Fi summary strip.
- Five high-contrast category cards:
  - Reader
  - Productivity
  - Games
  - Tools
  - Settings
- GPIO0 BOOT long-press hierarchical Back action.
- Inter and Atkinson Hyperlegible UI families with Compact, Standard and Large size profiles.
- Persistent display preferences from `/sdcard/RUSTMIX/DISPLAY.TXT`.

### Applications and device screens

| Category | Screen | Status |
| --- | --- | --- |
| Reader | Continue Reading, Library, Bookmarks | Placeholder routes |
| Productivity | Calendar | Ready: RTC-localized, read-only monthly view |
| Productivity | Voice Notes | Placeholder route; microphone capture deferred |
| Games | TBD | Placeholder route |
| Tools | File Browser | Ready: bounded read-only SDMMC browser and text preview |
| Tools | Dictionary | Placeholder route |
| Tools | Unit Converter | Ready: offline fixed-point Length, Mass, Temperature and Volume conversions |
| Settings | Alarms | Ready: RTC schedules, runtime edits, snooze and dismiss |
| Settings | Audio | Ready: ES8311 playback diagnostics and audible chime |
| Settings | Clock | Ready: localized RTC and power details |
| Settings | Display | Ready: UI font family and size persistence |
| Settings | Device Info | Ready: paginated firmware and board details |
| Settings | Environment | Ready: SHTC3 temperature and humidity |
| Settings | Motion | Ready: QMI8658 accelerometer and gyroscope |
| Settings | Network | Ready: SD-provisioned Wi-Fi and SNTP status |
| Settings | Weather | Ready with retries: Open-Meteo conditions and four-day forecast |

### Sleep-image mode

A short AXP2101 Power-key press enters network-suspended sleep-image mode:

1. Stop audio playback.
2. Select a hardware-random `800 × 480`, 1-bpp BMP from `/sdcard/RUSTMIX/SLEEP`.
3. Avoid immediately repeating the prior image when multiple valid BMP files exist.
4. Stop SNTP, disconnect Wi-Fi and pause Weather.
5. Render the sleep image, deep-sleep the e-paper panel and disable ALDO3.
6. Suppress stale PMIC wake events for a `900 ms` quiet window.
7. Restore the active route and reconnect services after a deliberate second Power-key press.

The MCU event loop intentionally remains active so Power-key polling and GPIO45 RTC-alarm readiness continue to work.

## Hardware summary

| Function | Interface / GPIO |
| --- | --- |
| E-paper SCLK / MOSI / CS / DC / RST / BUSY | GPIO11 / GPIO12 / GPIO10 / GPIO9 / GPIO46 / GPIO3 |
| PMIC and board-service I2C | SDA GPIO41, SCL GPIO42 |
| Application buttons | UP GPIO4, SELECT GPIO5, DOWN GPIO6 |
| BOOT Back action | GPIO0, active low, long press |
| SDMMC | CLK GPIO16, CMD GPIO17, D0 GPIO15, D1 GPIO7, D2 GPIO8, D3 GPIO18 |
| RTC alarm interrupt | GPIO45, active low |
| Audio TX | MCLK GPIO13, BCLK GPIO14, WS GPIO47, DOUT GPIO48 |
| Audio amplifier enable | GPIO39 |

`GPIO3` is reserved for e-paper BUSY and must not be reused as an application input.

## Repository layout

```text
src/
├── app/                  # Product routes, state, typography, widgets and screens
├── audio/                # ES8311 codec profile, I2S runtime and tone generator
├── calendar.rs           # Hardware-independent Gregorian calendar math
├── unit_converter.rs     # Offline fixed-point conversion domain
├── weather.rs            # Open-Meteo parser, retry policy and last-known-good cache
├── sleep_images.rs       # FAT-safe BMP scan and hardware-random anti-repeat selector
├── sleep_network.rs      # Wi-Fi / SNTP / Weather suspension state
├── power_key.rs          # AXP2101 Power-key policy and sleep-entry wake guard
└── main.rs               # ESP-IDF wiring and event-loop orchestration
```

## Build prerequisites

The firmware uses the Rust-on-ESP-IDF `std` workflow. On macOS:

```bash
xcode-select --install
brew install cmake ninja dfu-util libusb python3
```

Install Rust, the ESP Rust toolchain and ESP flashing utilities according to your local ESP-Rust setup. The repository expects:

```text
Rust toolchain: esp
Target: xtensa-esp32s3-espidf
ESP-IDF: v5.4.3
```

## Validate, test, build and flash

```bash
./scripts/validate.sh
./scripts/test-host.sh
source "$HOME/export-esp.sh"
./scripts/build.sh
./scripts/flash.sh /dev/cu.usbmodem2101
```

`./scripts/validate.sh` performs format and repository-contract checks. `./scripts/test-host.sh` runs hardware-independent library tests on the stable host toolchain. `./scripts/build.sh` performs the ESP release build.

## SD-card setup

Copy and edit the example configuration files:

```bash
./scripts/install-sd-examples.sh /Volumes/YOUR_SD_CARD
```

The installer does not overwrite existing files unless `--force` is supplied.

Required runtime paths:

```text
/RUSTMIX/WIFI.TXT
/RUSTMIX/WEATHER.TXT
/RUSTMIX/ALARMS.TXT
/RUSTMIX/DISPLAY.TXT
/RUSTMIX/SLEEP/*.BMP
```

See [`docs/SD_CARD_SETUP.md`](docs/SD_CARD_SETUP.md).

## Known issue: Weather provider reliability

The device-side retry and stale-cache behavior is implemented, but physical tests observed intermittent Open-Meteo `502`, TLS EOF and HTTP timeout failures. The Weather screen may remain unavailable until the provider path succeeds at least once during the active runtime session. See [`docs/KNOWN_ISSUES.md`](docs/KNOWN_ISSUES.md).

## Documentation

- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/BOARD_CONTRACT.md`](docs/BOARD_CONTRACT.md)
- [`docs/SD_CARD_SETUP.md`](docs/SD_CARD_SETUP.md)
- [`docs/PHYSICAL_SMOKE_TEST.md`](docs/PHYSICAL_SMOKE_TEST.md)
- [`docs/KNOWN_ISSUES.md`](docs/KNOWN_ISSUES.md)
- [`docs/REFERENCE_AUDIT.md`](docs/REFERENCE_AUDIT.md)
- [`docs/GITHUB_UPLOAD.md`](docs/GITHUB_UPLOAD.md)
- [`docs/CLEANUP_REPORT.md`](docs/CLEANUP_REPORT.md)

## Packaging a clean source ZIP

```bash
./scripts/package-release.sh
```

The generated archive is written under `dist/` and excludes build outputs, Git metadata and local caches.

## Licenses

Firmware source is MIT licensed. Generated embedded bitmap atlases are derived from Inter and Atkinson Hyperlegible under the SIL Open Font License 1.1. See [`docs/licenses/FONT_NOTICES.md`](docs/licenses/FONT_NOTICES.md).
