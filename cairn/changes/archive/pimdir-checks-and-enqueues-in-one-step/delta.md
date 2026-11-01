---
cairn: change
change: pimdir-checks-and-enqueues-in-one-step
---

# Delta

## MODIFIED Requirements

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`enqueue_with_body_checked`), which hashes it under the algorithm the store records in `store_meta.hash_algo`, and SHALL NOT hash or write a body itself. A body named under the wrong algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. A write carrying a body goes through the producer's own `enqueue_with_body_checked`, which checks the action, then writes the body under its staging lock and appends the row pinning it in one transaction, so a refused write leaves no body behind, and answers the partial supports the gate passed it with, each logged as a warning. The action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A pimdir write reaches a synced address book or is refused
The pimdir backend SHALL refuse `create_card`, `update_card` and `delete_card`, before anything reaches the blob tree or the queue, on an address book no source syncs: one for which the store records neither a source checkpoint nor a binding (`collection_sources`). The owner applies a queued `add` there as whichever source drains first, binding it to a collection that source never lists, so the card would never be pushed; an `update` of a card no source binds applies to the store alone and a `remove` retains the card, neither reaching a server. A collection the device keeps for itself is the owner's to write (pimdir STORAGE §14.3), never a producer's.

### Requirement: A duplicate pimdir create is refused
`create_card` on pimdir SHALL refuse a card whose link id the address book already holds, as a live item or a pending create, appending nothing to the queue and writing nothing to the blob tree, with the code `item-duplicate`. The refusal SHALL be io-pimdir's (`PimdirProducer::enqueue_with_body_checked`, checked in the transaction that would append the row, before the body is written), never a lookup of cardamum's own. The owner would park such an `add` at drain (pimdir STORAGE §15.3), so accepting it reports a success no sync carries out. With `-i`, the composer file is kept and named, as on any failed write.

### Requirement: account check opens every role the backend takes
`account check` on pimdir SHALL open the store as a reader and list its address books, then open a producer and take its staging lock, so a store whose staging lock cannot be taken fails the check rather than the first write. A producer takes the lock at its first staged body, not at its open, so the check opens a blob writer and drops it uncommitted, which removes its temporary file and queues nothing.
