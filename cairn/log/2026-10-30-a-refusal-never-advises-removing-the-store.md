---
cairn: log
change: a-refusal-never-advises-removing-the-store
landed: 2026-10-30
---

# A refusal never advises removing the store

`delete_addressbook` on pimdir (src/pimdir/backend.rs) now says "The pimdir backend cannot delete an addressbook; delete it on the server and sync", dropping the advice to remove the store directory, which loses the queue. The `store-busy` code reads "another writer held the pimdir store, past the wait on a write and at once on `account check`, which does not wait; retry", the check's producer probe failing without waiting; `ErrorCode::StoreBusy` documents the same.

No test: wording only, the refusal itself is unchanged.

Capabilities moved: **backends** (pimdir addressbook limits are explicit, modified), **commands** (A JSON error carries a stable code, modified).
