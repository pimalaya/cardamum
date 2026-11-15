---
cairn: tasks
change: a-report-carries-its-errors
---

# Tasks

- [x] `CarddavCardEntriesOutput` gains `errors: Vec<ItemError>`, its text printing them through `ItemErrors` after the table and the truncation line.
- [x] `report query` maps `CarddavCardListOk::unreadable` through `unreadable_row`; `report multiget` maps `CarddavCardMultigetOk::unreadable`, a 404 or 410 row `card-not-found`, the others through `unreadable_row`; both end with `partial_when`.
- [x] The `--help` JSON lines name `errors`.
- [x] Tests over a scripted multistatus (src/carddav/report/entries.rs): a query with a refused row, a refused `address-data`, a row without data and a 507 row; a multiget with a refused row and a 404 row; a multiget reading every card; the text with and without rows. The two command tests fail with the entries left out.
- [x] CHANGELOG breaking entry, MIGRATION.md, the S4 history line of cairn/spec/testing/carddav-specific.md.
- [x] Fold the delta into commands.md and backends.md, write the log entry, set `landed`, archive.
