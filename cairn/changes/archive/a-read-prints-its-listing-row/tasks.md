---
cairn: tasks
change: a-read-prints-its-listing-row
---

# Tasks

- [x] src/shared/card/read.rs: `CardReadOutput` = `CardRow` + `CardContents`; src/shared/card/build.rs on `CardContents`.
- [x] Tests: the read prints its row's keys and values; a card not UTF-8 reads as base64 (fails on the old code, which failed the read).
- [x] Fold the delta, log, CHANGELOG, MIGRATION.md.
