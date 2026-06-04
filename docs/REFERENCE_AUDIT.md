# Waveshare reference audit summary

This Rust project was built from a narrow audit of the Waveshare ESP-IDF sample application and then split into reviewed modules.

## Adopted board facts

- SSD1677-compatible 800 × 480 e-paper transport and panel power sequencing.
- AXP2101 `ALDO3` ownership for the panel rail.
- Shared I2C bus on GPIO41 / GPIO42.
- SDMMC 4-bit wiring and FAT mount at `/sdcard`.
- PCF85063 RTC alarm register layout and sticky alarm-flag behavior.
- SHTC3 wake / measure / sleep sequence with CRC validation.
- QMI8658 probe and contiguous diagnostics reads.
- ES8311 playback wiring with GPIO48 as MCU-to-codec TX and GPIO39 amplifier enable.
- AXP2101 PEK short-press interrupt handling for the physical Power key.

## Intentional differences

- The Rust firmware uses modular source files instead of a large translated application unit.
- Weather uses removable-SD latitude / longitude configuration and Open-Meteo HTTPS rather than an embedded provider key.
- File Browser behavior is bounded and read-only.
- Global UI typography uses generated 1-bpp bitmap atlases rather than shipping raw font files.
- Sleep images are hardware-random with immediate-repeat avoidance.
- Network services pause while the sleep image is visible.
- MCU deep sleep remains deferred until it can be validated independently.
