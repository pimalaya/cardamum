---
cairn: tasks
change: pimdir-status-shows-parked-actions
---

# Tasks

- [x] `pimdir` command family (src/pimdir/cli.rs) and `pimdir status` (src/pimdir/status.rs), wired into src/cli.rs behind the `pimdir` feature; `PimdirBackend::status` (src/pimdir/backend.rs) narrowed to the account read.
- [x] `cardamum-pimdir-status` in src/json_schema.rs.
- [x] Test: a guarded edit the owner parks (a sync replaced the body after the edit was queued, then drain) shows in `status` with its row and its reason; another account's parked row does not.
- [x] `pimdir status` help and JSON description; src/main.rs and src/pimdir/mod.rs headers.
- [x] CHANGELOG Added entry; README.
- [x] Fold the delta into [commands](../../spec/commands.md) and [backends](../../spec/backends.md); log; land.
