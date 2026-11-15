---
cairn: log
change: a-read-prints-its-listing-row
date: 2026-11-15
---

# A read prints its listing row

`card read` prints its `card list` row (`addressbookId`, `fnValue`, `email`, `tel` join) plus the vCard, `contents` a string, the text when UTF-8 (`encoding` `utf-8`) and the base64 otherwise (`encoding` `base64`); a card that is not UTF-8 reads where it failed uncoded, projecting no common field rather than a lossy one. `card build` carries its card the same way. Tasks C12 and C13 of cli-alignment-2026-11-15 (findings F9, F14, decision D7). Capability **commands** moved.

The `encoding` key is calendula's (its K4, the same day), so the twins print bytes alike; output.md asks for a string, base64 when not UTF-8, and no field changing type between outputs. base64-codec is no longer jmap's alone. Evidence: src/shared/card/read.rs tests (the row's keys and values, the base64 card). MIGRATION.md and CHANGELOG say it.
