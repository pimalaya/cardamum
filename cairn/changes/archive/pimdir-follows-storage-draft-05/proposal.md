---
cairn: change
id: pimdir-follows-storage-draft-05
status: landed
created: 2026-10-30
---

# The pimdir backend follows storage-draft-05

## Why

io-pimdir landed the-store-version-contract and the-store-follows-storage-draft-05 on 2026-10-30 (local checkout, path-pinned), and cardamum no longer builds against it.

- **A queued edit overwrites what a sync brought in after it was staged.** `PimdirAction::Update` gains `if_match`: the body the producer read and replaced. `update_card` gates the staging on `--if-match` (or the version `card update` read), but the row it queues names no base, so a server change a sync pulls between the staging and the drain is replaced by the edit without a merge (src/pimdir/backend.rs:255).
- **A refused write leaves its body behind.** `stage` writes the body with `stage_body` before `enqueue_add` checks the link id, so a create refused as `item-duplicate` leaves an unpinned body in objects/ for the collector (the finding pimdir-release-api-followups recorded). `PimdirProducer::enqueue_with_body` checks first and writes the body inside the enqueue transaction.
- **New store refusals.** `PimdirError::Foreign` (`store-foreign`) is raised by a reader or a producer opening a database another program wrote; cardamum maps none of it. `Queued` and `Role` come from collection deletes and role writes, which cardamum never makes. A reader now opens a store older than its crate's version, so `store-older` is no longer raised on any cardamum path.

## What

- `update_card` states as `if_match` the version the write was gated on, the body the caller read, when `--if-match` (or the read of `card update`) named one; `None` when the caller read nothing (a vCard source and no `--if-match`). The owner then parks an edit whose base the item no longer holds, and the parked row is the sync engine's to report.
- `stage` goes through `enqueue_with_body`; nothing else stages a body.
- `ErrorCode::StoreForeign` (`store-foreign`); `ErrorCode::StoreOlder` dropped, never released and no longer reachable.
- Tests failing first: an edit whose read body a sync replaced before the drain is parked, not applied; a refused duplicate create leaves objects/ as it was.
