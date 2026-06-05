# Known issues and deferred work

## Weather provider reliability

Physical testing has observed intermittent Open-Meteo failures:

```text
HTTP 502
TLS EOF
HTTP response timeout
ESP_ERR_HTTP_CONNECT
```

The device-side resilience layer is active:

- initial request plus three retries
- event-loop scheduled `2`, `5`, and `15` second backoff
- retry handling for TLS, transport, timeout and HTTP `429`, `500`, `502`, `503`, `504`
- last-known-good in-memory forecast retention
- `RETRYING` and `STALE` UI states

When no successful fetch has occurred since boot, Weather may still show `Weather unavailable` after retries are exhausted. This issue is currently on hold while provider-path behavior is observed.

## MCU deep sleep

Sleep-image mode powers down the panel rail and pauses network services, but the MCU event loop remains active. This preserves PMIC Power-key polling and the physically validated GPIO45 RTC-alarm path. MCU deep sleep is deferred to a separate milestone.

## Voice Notes

Audio playback is active, but microphone RX capture is deferred. `Productivity > Voice Notes` remains a placeholder.

## Deferred applications and Reader work

Reader TXT and bounded reflowable EPUB opening, Continue Reading, Recent, bookmarks, Reader Preferences and EPUB TOC navigation are active. Deferred Reader work includes EPUB CSS layout, images, hyperlinks, footnotes, fixed-layout EPUB, DRM and SD-backed EPUB anchor caches. The remaining placeholder apps are:

- Dictionary
- Games TBD
