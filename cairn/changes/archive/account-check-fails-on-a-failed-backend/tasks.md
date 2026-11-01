---
cairn: tasks
change: account-check-fails-on-a-failed-backend
---

# Tasks

- [x] Test: `account check` over a vdir home that is missing fails with an error naming the backend and why; fails on the old exit 0.
- [x] Return the error from `AccountCheckCommand::execute` (src/account/check.rs).
- [x] Mark V9 fixed in [vdir-local](../../../spec/testing/vdir-local.md) and [testing/README](../../../spec/testing/README.md).
- [x] CHANGELOG Fixed entry.
- [x] Fold the delta into [commands](../../../spec/commands.md); log; land; archive.
