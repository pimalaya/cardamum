---
cairn: change
change: pimdir-follows-storage-draft-05
---

# Delta

## ADDED Requirements

### Requirement: A queued pimdir edit names the body it replaced
`update_card` on pimdir SHALL state on the queued `update` the body the edit was made against (`if_match`, pimdir STORAGE §15.3): the version `--if-match` named, or the one `card update` read when given no vCard source, and SHALL state none only when the caller read no body. The owner parks a row whose stated body the item no longer holds, so a server change a sync brings in between the staging and the drain is not overwritten without a merge. The parked row is the sync engine's to report: cardamum's write has already said it is queued.

## MODIFIED Requirements

### Requirement: pimdir derivations match the sync engine
The link id, summary and sort key of a card a pimdir write stages SHALL be io-pimdir's own derivations (`io_pimdir::summary`, pimdir STORAGE Annex A), and the backend SHALL carry none of its own. A queued `add` or `update` names the body alone: the store's owner derives the summary and the sort key from it when it applies the action, so a card Cardamum stages summarizes exactly as the same card arriving through a sync. The link id `create_card` reports is the same derivation run on the same bytes: the bare `UID`, or `esc:` and the `UID` when it begins with a prefix the store reserves for the keys it derives (`alt:`, `hash:`, `dup:`, `part:`, `file:`, `esc:`, pimdir STORAGE §9), or `hash:` over the body when the card states none.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir STORAGE §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`enqueue_with_body`), which hashes it under the algorithm the store records in `store_meta.hash_algo`, and SHALL NOT hash or write a body itself. A body named under the wrong algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. A write carrying a body goes through the producer's own `enqueue_with_body`, which checks the action, then writes the body under its staging lock and appends the row pinning it in one transaction, so a refused write leaves no body behind. The action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A duplicate pimdir create is refused
`create_card` on pimdir SHALL refuse a card whose link id the address book already holds, as a live item or a pending create, appending nothing to the queue and writing nothing to the blob tree, with the code `item-duplicate`. The refusal SHALL be io-pimdir's (`PimdirProducer::enqueue_with_body`, checked in the transaction that would append the row, before the body is written), never a lookup of cardamum's own. The owner would park such an `add` at drain (pimdir STORAGE §15.3), so accepting it reports a success no sync carries out. With `-i`, the composer file is kept and named, as on any failed write.

### Requirement: A store refusal carries its code
The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-newer`, `store-foreign`, `store-stale`, `store-corrupt`, `store-hash-algo`, `store-busy`, `unsupported` and `item-duplicate`; an SQL, I/O or JSON failure carries none, and so does a code no cardamum path raises. A store older than the io-pimdir cardamum builds against opens: a reader and a producer read it as it is. The message is io-pimdir's and SHALL NOT advise deleting the store: a store this cardamum cannot read asks for its sync engine to run or be upgraded, or for cardamum's upgrade, and a resync restores neither the queue nor what the store keeps alone.

### Requirement: A JSON error carries a stable code
The table of codes loses `store-older`, never released and no longer raised, and gains one row:

| Code | Raised when |
|---|---|
| `store-foreign` | the pimdir root holds a database another program wrote |

## REMOVED Requirements
