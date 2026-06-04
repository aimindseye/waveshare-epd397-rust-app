# Board contract

The firmware targets the Waveshare ESP32-S3 3.97-inch e-paper board with a native `800 × 480` monochrome panel and a logical `480 × 800` portrait product UI.

## GPIO and bus ownership

| Function | GPIO / interface | Notes |
| --- | --- | --- |
| E-paper SCLK | GPIO11 | SPI |
| E-paper MOSI | GPIO12 | SPI |
| E-paper CS | GPIO10 | SPI |
| E-paper DC | GPIO9 | Output |
| E-paper RST | GPIO46 | Output |
| E-paper BUSY | GPIO3 | Input; do not reuse |
| Shared I2C SDA | GPIO41 | PMIC, RTC, SHTC3, QMI8658, ES8311 |
| Shared I2C SCL | GPIO42 | 400 kHz |
| UP | GPIO4 | Active low |
| SELECT | GPIO5 | Active low |
| DOWN | GPIO6 | Active low |
| BOOT Back | GPIO0 | Active-low long press |
| SDMMC CLK | GPIO16 | 4-bit SDMMC |
| SDMMC CMD | GPIO17 | 4-bit SDMMC |
| SDMMC D0 | GPIO15 | 4-bit SDMMC |
| SDMMC D1 | GPIO7 | 4-bit SDMMC |
| SDMMC D2 | GPIO8 | 4-bit SDMMC |
| SDMMC D3 | GPIO18 | 4-bit SDMMC |
| RTC alarm INT | GPIO45 | PCF85063 active-low route |
| ES8311 MCLK | GPIO13 | I2S TX |
| ES8311 BCLK | GPIO14 | I2S TX |
| ES8311 WS / LRCLK | GPIO47 | I2S TX |
| ES8311 DOUT | GPIO48 | MCU to codec |
| ES8311 DIN | GPIO21 | Deferred RX capture |
| Audio PA enable | GPIO39 | Shared board ownership; used by audio runtime |

## Service addresses

| Service | Address / behavior |
| --- | --- |
| AXP2101 PMIC | Shared I2C power and Power-key IRQ handling |
| PCF85063 RTC | `0x51` |
| SHTC3 | `0x70` |
| QMI8658 | Probe `0x6A`, then `0x6B`; require `WHO_AM_I = 0x05` |
| ES8311 | 7-bit `0x18`, then `0x19` fallback |

## Stable constraints

- Keep `GPIO3` reserved for `EPD_BUSY`.
- Keep SDMMC optional and read-only for File Browser operations.
- Keep audio startup muted and amplifier disabled until explicit playback.
- Keep Wi-Fi startup after the first panel frame so networking cannot block visible boot.
- Keep MCU deep sleep out of scope until a separate GPIO45 wake milestone is physically validated.
