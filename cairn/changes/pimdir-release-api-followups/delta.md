---
cairn: change
change: pimdir-release-api-followups
---

# Delta

## ADDED Requirements

### Requirement: A duplicate pimdir create is refused
`create_card` on pimdir SHALL refuse a card whose link id the address book already holds, as a live item or a pending create, appending nothing to the queue, with the code `item-duplicate`. The refusal SHALL be io-pimdir's (`PimdirProducer::enqueue_add`, checked in the transaction that would append the row), never a lookup of cardamum's own. The owner would park such an `add` at drain (pimdir STORAGE §15.3), so accepting it reports a success no sync carries out. The body staged before the refusal is pinned by nothing and the store's collector removes it. With `-i`, the composer file is kept and named, as on any failed write.

### Requirement: A store refusal carries its code
The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-newer`, `store-older`, `store-stale`, `store-corrupt`, `store-hash-algo`, `store-busy`, `unsupported` and `item-duplicate`; an SQL, I/O or JSON failure carries none. The message is io-pimdir's and SHALL NOT advise deleting the store: a store this cardamum cannot read asks for its sync engine to run or be upgraded, or for cardamum's upgrade, and a resync restores neither the queue nor what the store keeps alone.

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add` through `enqueue_add`, `update_card` to `update`, `delete_card` to `remove`. The body reaches the blob tree through the producer's own `stage_body`, under its staging lock, before the row that pins it is appended, and the action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`stage_body`), which hashes it under the algorithm the store records in `store_meta.hash_algo`, and SHALL NOT hash or write a body itself. A body named under the wrong algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: A JSON error carries a stable code
A failure a caller is expected to act on SHALL carry a stable code, printed under `--json` as a `code` string beside `error`, `sources` and `backtrace`. The code SHALL be found anywhere in the error chain, so added context does not hide it. A failure with no code SHALL print no `code` field. The plain output SHALL stay unchanged. Codes are kebab-case and never renamed; `error` stays free wording. A pimdir store's code is the one io-pimdir names (`PimdirError::code()`), never read off its message.

| Code | Raised when |
|---|---|
| `body-pending` | a listed card's body is not local yet (pimdir); a sync brings it |
| `precondition-failed` | a write's `--if-match` names a version the card no longer has (pimdir) |
| `store-uncreated` | the pimdir root holds no store; its sync engine creates it |
| `store-newer` | the pimdir store is newer than this cardamum reads; upgrade cardamum |
| `store-older` | the pimdir store is older than this cardamum reads; its sync engine migrates it |
| `store-stale` | the pimdir store lacks part of its schema; run or upgrade its sync engine |
| `store-corrupt` | the pimdir store's two version stamps disagree |
| `store-hash-algo` | the pimdir store names its bodies by a hash this cardamum does not know |
| `store-busy` | another writer held the pimdir store past the wait; retry |
| `unsupported` | a source syncing the address book does not support the write (pimdir STORAGE §15.6); nothing was queued |
| `item-duplicate` | a create names a `UID` the pimdir address book holds, synced or queued; nothing was queued |

## REMOVED Requirements

### Requirement: A store from an earlier draft is refused
Replaced by "A store refusal carries its code": the fix it named, deleting the store, loses what a resync cannot restore.
