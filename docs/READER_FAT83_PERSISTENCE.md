# Reader FAT 8.3 persistence filename repair

RustMix Wave v0.16.6 repairs Reader-owned writable filenames for FAT filesystems where long-name creation is unavailable or unreliable.

## Short-name-safe runtime layout

```text
/RUSTMIX/READER/
├── STATE.TXT
├── POSITS.TXT
├── RECENT.TXT
├── MARKS.TXT
├── PREFS.TXT
└── CACHE/
    └── ED9B69AF.CCH
```

Atomic replacement siblings use `.TMP` and `.BAK`. All generated writable filenames have a basename of eight characters or fewer and an extension of three characters or fewer.

## Legacy migration

When `POSITS.TXT` and `POSITS.BAK` are absent, the Reader attempts a read-only load from legacy `POSITIONS.TXT` or `POSITIONS.BAK`. Valid records are migrated into `POSITS.TXT`. New writes never target the legacy long filename.

## Cache collision safety

Cache basenames are deterministic eight-character hexadecimal values. The cache payload fingerprint remains authoritative: path, source size, modified timestamp, format and layout preferences must match before anchors are accepted. A collision therefore becomes a safe cache miss followed by lazy rebuilding.

## v0.16.7 runtime completion

All writable call sites are audited. Positions write to `POSITS.TXT`, `POSITS.TMP` and `POSITS.BAK`. TXT anchor caches write to `<8HEX>.CCH`, `<8HEX>.TMP` and `<8HEX>.BAK` without an extra `B` prefix. Legacy `POSITIONS.TXT` remains read-only migration input. Repeated identical degraded log lines are suppressed until status changes.
