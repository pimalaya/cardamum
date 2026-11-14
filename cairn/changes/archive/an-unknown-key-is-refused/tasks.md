---
cairn: tasks
change: an-unknown-key-is-refused
---

# Tasks

- [x] `deny_unknown_fields` on `Config` and `AccountConfig`; no `flatten` in the config.
- [x] The removed tables refused by name, pointing at MIGRATION.md.
- [x] A compiled-out backend block loads unread.
- [x] Tests: an unknown key at the root and in an account, a removed table by name (each failing on the old code), a backend and nested table (already refused), a backend left out of the build; the sample still loads in every variant.
- [x] CHANGELOG breaking entry, MIGRATION.md.
- [x] Fold the delta into config.md, write the log entry, set `landed`.
