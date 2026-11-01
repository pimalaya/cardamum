---
cairn: tasks
change: a-refusal-never-advises-removing-the-store
---

# Tasks

- [x] Drop "or remove the store directory" from `delete_addressbook` (src/pimdir/backend.rs).
- [x] Reword `ErrorCode::StoreBusy` (src/error.rs) to cover the check's non-waiting probe.
- [x] Quote the new message in [pimdir-local](../../../spec/testing/pimdir-local.md).
- [x] Fold the delta into [backends](../../../spec/backends.md) and [commands](../../../spec/commands.md); log; land; archive.
