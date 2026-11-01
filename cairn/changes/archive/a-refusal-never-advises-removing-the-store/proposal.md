---
cairn: change
id: a-refusal-never-advises-removing-the-store
status: landed
created: 2026-10-30
---

# A refusal never advises removing the store

## Why

The pimdir stable-release plan of 2026-10-30 (Apps, Phase A, Common) forbids advising to delete the store: it loses the queued writes a resync never restores, and [backends](../../../spec/backends.md) already says a store refusal SHALL NOT advise it. Yet `addressbook delete` on pimdir still says "delete it on the server and sync, or remove the store directory" (src/pimdir/backend.rs, `delete_addressbook`), and the spec and the pimdir test report repeat it.

The `store-busy` row of the JSON code table says another writer "held the pimdir store past the wait", but `account check` probes the producer without waiting (a-check-probes-the-producer), so a busy check fails at once.

## What

- `delete_addressbook` names the server route only: "The pimdir backend cannot delete an addressbook; delete it on the server and sync".
- [backends](../../../spec/backends.md) "pimdir addressbook limits are explicit" names the same route and forbids the store-removal advice; the pimdir test report quotes the new message.
- The `store-busy` row, and the `ErrorCode::StoreBusy` doc the JSON schema carries, cover both: past the wait on a write, at once on `account check`.
