---
cairn: tasks
change: pimdir-follows-storage-draft-09
---

# Tasks

- [x] Pin io-pimdir 5c18a02, requirement 0.9; drop the ical-rs patch; vcard-rs 0.6
- [x] Version tests stage the previous drafts (`storage-draft-08`, `search-draft-08`), each failing against the 0c70144 pin
- [x] `cargo test --all-features`, clippy, the feature matrix
- [x] Fold into spec/backends.md and spec/commands.md, CHANGELOG; log
- [ ] End to end on a scratch store a draft-09 neverest synced (supervisor)
- [x] Land
