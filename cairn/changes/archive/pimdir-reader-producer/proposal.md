---
cairn: change
id: pimdir-reader-producer
status: landed
created: 2026-08-28
---

# pimdir: Cardamum holds the one handle it must not hold

io-pimdir 0.3 names the three roles the format defines (spec §8) and gives each one a handle. `PimdirStore` is the **owner**: it drains the queue, sweeps the objects, purges the trash, and now takes an exclusive advisory lock on the store directory for its whole lifetime, a second owner getting `PimdirError::Owned` immediately. `PimdirReader` is the **read** surface, which takes no lock and carries no write at all. `PimdirProducer` is the **enqueue-only** handle for a process that originates mutations without owning the store.

Cardamum opens the owner. It reads a store Neverest syncs and it stages a handful of item mutations, which is a reader plus a producer exactly, and instead it holds the handle that can destroy the store and locks the sync out while it prints a contact list. Since the lock landed, `cardamum card list` run while a sync is in flight fails outright, and a sync started while Cardamum is running fails the same way.

The backend does not compile against 0.3 either. `PimdirStore::open` lost its source parameter, the storage seam moved onto `PimdirSourceStore` (`load_hub`, `for_source`), and `ReplicaYield::WantsLoad` became a struct variant carrying a `ReplicaLoadScope`.

Two defects sit underneath, both invisible while the code did not build:

**The link ids disagree.** Cardamum derives `uid:<UID>`; Neverest and io-pimdir's own conventions derive the bare `UID`. The store this backend reads is on the bare form, so a card Cardamum added would link as a second identity, store a second body, and sync as a second contact. The requirement that the two agree byte for byte has been false since the format settled the id.

**The content hash is not FNV-1a.** The spec here still says the body hash is "the shared 128-bit FNV-1a digest rendered as 32 hex chars". A store names its bodies by the algorithm recorded in `store_meta.hash_algo`, and the live Posteo store names them in base32. A writer that hashes by hand under its own algorithm writes a body no read ever finds.

## What changes

- The backend holds a `PimdirReader` built `with_pending`, opened read-only, taking no lock. Reads run beside a sync, and an action Cardamum staged shows in its own listing before the owner applies it.
- A write opens a `PimdirProducer` for the length of that one write and drops it: the body goes to the blob tree through `PimdirBlobs::writer`, then one queue row. `create_card` enqueues `add`, `update_card` `update`, `delete_card` `remove`, each addressing the item by the public `seq` that is already Cardamum's card id.
- `pimdir.source` goes away. A queued action is attributed to a producer name, not to a sync source, so there is nothing left to configure or auto-detect, and the class of failure the auto-detection existed to soften (a write staged against a source no sync drives) cannot arise.
- The vCard derivations delegate to `io_pimdir::conventions::card`, which is the format's own scanner. The link id becomes the bare `UID`, agreeing with the store. A queued create carries no sort key, the format leaving that to the sync that pushes it.
- The body hash comes from the store (`PimdirReader::hash`), never from a digest this crate picks.
- `create_addressbook` refuses. Declaring a collection is an owner write, and Cardamum is not the owner; a locally-declared collection is also one no sync would ever carry, which is the reason `update_addressbook` and `delete_addressbook` already refuse.
- `create_card` reports the card's link id rather than a `seq`, a queued create having no public id until the owner applies it.

## What does not change

The read semantics the cache requirements pin: an unhydrated card still lists as a preview projected from its `v: 1` summary and still refuses to be read by `get_card`, an unknown addressbook still fails rather than reading as empty, and the store path is still shell-expanded before anything opens it.
