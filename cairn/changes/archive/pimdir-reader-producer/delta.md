---
cairn: change
id: pimdir-reader-producer
status: landed
created: 2026-08-28
---

# Delta

## ADDED Requirements

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`. The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them. The reader SHALL be built with the pending overlay, so an action this process staged reads back before the store's owner applies it.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL name a body it writes with the hash the store records in `store_meta.hash_algo`, read through the handle it holds, and SHALL NOT compute a digest of its own choosing. A body named under the wrong algorithm is a body no read ever finds.

## MODIFIED Requirements

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir SPEC §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. The body reaches the blob tree through the blob writer before the row that pins it is appended, and the action addresses the item by the public `seq` that is already the card's shared id. `update_card` ignores `--if-match`, because the engine reconciles the applied edit against the base body it recorded at sync time, which is stronger than an ETag precondition a local store cannot check. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

### Requirement: pimdir derivations match the sync engine
The link id and the `v: 1` summary a pimdir write records SHALL be derived by `io_pimdir::conventions::card`, the format's own derivations, so a card Cardamum stages links and summarizes exactly as the same card arriving through a sync. The link id is the bare `UID`, with nothing prepended. A queued action carries no sort key: the format leaves the key to the sync that pushes the create, and a producer deriving one would order an item the connector is about to reorder.

### Requirement: pimdir addressbook limits are explicit
Every `addressbook` write SHALL fail on pimdir, each naming its reason. `create_addressbook` refuses because declaring a collection is an owner write and this backend is a producer, and because a collection no sync knows about is one no sync would carry. `update_addressbook` refuses because the collection row (id, display name, description, colour) is what a sync writes from the server, and io-pimdir's `rename_collection` renames the identifier rather than the display name. `delete_addressbook` refuses with a message naming the alternative (delete on the server and sync, or remove the store directory), because io-pimdir exposes no collection removal.

## REMOVED Requirements

### Requirement: pimdir writes auto-source
Removed: a queued action is attributed to a producer name rather than to a replica source, so there is no source to configure, none to auto-detect from the store, and no store-was-not-synced-as-this-source failure left to report. `pimdir.source` leaves the configuration with it.
