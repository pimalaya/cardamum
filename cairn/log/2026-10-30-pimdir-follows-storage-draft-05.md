---
cairn: log
change: pimdir-follows-storage-draft-05
landed: 2026-10-30
---

# The pimdir backend follows storage-draft-05

cardamum builds against io-pimdir's the-store-version-contract and the-store-follows-storage-draft-05 (2026-10-30, the local checkout still path-pinned; Cargo.lock unchanged by this change). Capabilities **backends** and **commands** moved.

## What landed

- **A queued edit names the body it replaced** (src/pimdir/backend.rs, `update_card`): the `update` row states as `if_match` the version `--if-match` named, or the one `card update` read when given no vCard source, and none when the caller read nothing. The owner parks a row whose stated body a sync replaced before the drain, instead of overwriting that server change unmerged. cardamum has no view of parked rows: the sync engine reports them.
- **A refused write leaves no body** (src/pimdir/backend.rs, `stage`): the body and the row go through `PimdirProducer::enqueue_with_body`, which runs the §15.6 gate and the duplicate check before writing the body, in the enqueue transaction. The `stage_body` call and the `enqueue_add` dispatch are gone. Closes the unpinned-body note of pimdir-release-api-followups.
- **`store-foreign`** maps io-pimdir's `PimdirError::Foreign`, which a reader or a producer raises on a database another program wrote; its message advises nothing destructive. `store-older` is dropped, never released: a reader and a producer now open an older store. `Queued` (`collection-queued`) and `Role` (`collection-role`) come from calls cardamum never makes and carry no cardamum code; `ErrorCode::of_pimdir` matches on `code()` with a wildcard, so nothing hides a variant.
- **The reported link id may carry `esc:`**: io-pimdir now files a `UID` beginning with a reserved prefix (`hash:`, `dup:`, ...) under `esc:` and the `UID`, and `card create` reports that key as it reports any link id. The spec's derivations requirement says so; no code changed.

## Tests

- `an_edit_whose_body_a_sync_replaced_is_parked`: on the port stating `if_match: None` the drain applied the edit (`PimdirDrainReport { applied: 1, parked: 0, .. }`); it now parks it and the card reads the server's body.
- `a_create_naming_a_held_uid_is_refused` asserts objects/ unchanged across the refused create; with `stage_body` first it held the refused body as a second file.
- `an_address_book_is_told_by_its_media_type` declares a bare `text/vcard`, the store now refusing a kind with parameters (`CHECK constraint failed: kind IN (...)`); the parameter case, which a store an earlier io-pimdir wrote may still carry, is checked on `is_card_kind`.
- 78 tests pass with every feature, 46 with pimdir alone; clippy is clean with every feature, the default ones, and each backend alone with `rustls-ring`.
