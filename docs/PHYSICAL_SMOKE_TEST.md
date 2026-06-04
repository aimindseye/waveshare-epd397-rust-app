# v0.15.0 physical smoke test

Run this checklist after flashing a release build.

## Boot and Home

- [ ] Firmware logs `version=0.15.0 milestone=unit-converter-foundation`.
- [ ] Home shows the simplified dark header.
- [ ] Localized date and time are visible.
- [ ] Weather, battery and Wi-Fi summary values are visible.
- [ ] Reader, Productivity, Games, Tools and Settings cards navigate correctly.

## Calendar

- [ ] Open `Productivity > Calendar`.
- [ ] Current localized date is highlighted.
- [ ] UP / DOWN moves through days in Day mode.
- [ ] SELECT toggles Month mode.
- [ ] Month navigation crosses a year boundary.
- [ ] Hold BOOT and confirm return to Productivity.

## Unit Converter

- [ ] Open `Tools > Unit Converter`.
- [ ] Confirm default `5 mi → 8.047 km` conversion.
- [ ] Verify `32 F → 0 C`.
- [ ] Verify `1 gal → 3.785 L`.
- [ ] Cycle `0.1`, `1`, `10` and `100` step sizes.
- [ ] Hold BOOT and confirm return to Tools.

## Settings and services

- [ ] Open Display and change UI family and size.
- [ ] Reboot and confirm `/RUSTMIX/DISPLAY.TXT` persistence.
- [ ] Open Audio and play the test chime.
- [ ] Open Alarms and confirm schedule list, snooze and dismiss paths.
- [ ] Open Environment and Motion and confirm sensor values update.
- [ ] Open Network and confirm Wi-Fi and SNTP status.

## Sleep-image mode

- [ ] Enter sleep from Home with one short Power-key press.
- [ ] Confirm a random BMP renders.
- [ ] Confirm stale entry-press event is suppressed during the `900 ms` quiet window.
- [ ] Leave the sleep image visible for at least five seconds.
- [ ] Press Power again and confirm Home restores.
- [ ] Repeat from Calendar and Unit Converter; confirm active-route restoration.
- [ ] Enter sleep twice and confirm consecutive images do not repeat when multiple valid BMP files exist.
- [ ] Confirm Wi-Fi, SNTP and Weather pause during sleep and resume after wake.

## Known Weather behavior

- [ ] If Open-Meteo fails, confirm non-blocking retry markers for `2`, `5` and `15` seconds.
- [ ] Confirm UI navigation remains responsive while a retry is pending.
