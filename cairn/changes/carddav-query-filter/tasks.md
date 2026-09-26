---
cairn: tasks
change: carddav-query-filter
---

# Tasks

- [x] Bump io-webdav to 0.4 and migrate `list_cards` callers (src/carddav/backend.rs, the shared client) to `CarddavCardListOptions::default()`.
- [x] src/carddav/report/query.rs: the filter flags, mapped onto `CarddavFilter`; short options checked against the global ones.
- [x] src/carddav/report/entries.rs: `truncated` in the output, an FN column read from the vCard.
- [x] CHANGELOG under [Unreleased].
- [ ] Live read-only queries on test accounts (Fastmail, Nextcloud, Radicale, iCloud, Google), recorded in cairn/spec/testing/carddav-specific.md.
- [ ] Fold the delta into cairn/spec/commands.md, write the log entry, mark `landed`.
