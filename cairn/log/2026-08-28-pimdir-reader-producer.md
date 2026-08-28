---
cairn: log
change: pimdir-reader-producer
landed: 2026-08-28
---

# pimdir: Cardamum stops holding the one handle it must not hold

io-pimdir 0.3 gives each of the format's three roles (spec §8) a handle of its own. `PimdirStore` is the owner: it drains the queue, sweeps the objects, purges the trash, and takes an exclusive advisory lock on the store for its whole lifetime. `PimdirReader` is the read surface, which takes no lock and carries no write at all. `PimdirProducer` is the enqueue-only handle for a process that originates mutations without owning the store.

Cardamum was opening the owner. It reads a store Neverest syncs and it stages a handful of card writes, which is a reader plus a producer exactly, and instead it held the handle that can destroy the store and locked the sync out for the length of a contact listing. Since the exclusive lock landed, the two could not run at the same time in either order.

The backend now holds a `PimdirReader` built `with_pending`, and opens a `PimdirProducer` for each write and drops it. `create_card` appends an `add`, `update_card` an `update`, `delete_card` a `remove`, each addressing the item by the public `seq` that was already the card's shared id. The body reaches the blob tree through the blob writer before the row pinning it is appended, and the producer is opened around the pair rather than around the enqueue alone, since its shared lock is what keeps a collector out of that window. Verified by holding `owner.lock` from another process and listing anyway.

`pimdir.source` is gone from the configuration. A queued action is attributed to a producer name, not to a replica source, so there is nothing left to configure, nothing to auto-detect from the store, and no "this store was not synced as that source" failure left to report. The requirement that described the auto-detection was removed rather than reworded.

**The link ids disagreed.** Cardamum derived `uid:<UID>`; Neverest and io-pimdir's own conventions derive the bare `UID`, and the live Posteo store is on the bare form. A card Cardamum staged would have linked as a second identity, stored a second body, and reached the server as a duplicate contact. The derivations delegate to `io_pimdir::conventions::card` now, so the agreement is one implementation rather than two kept in step by hand. The private scanner and its tests went with it.

**The body hash was Cardamum's own.** The spec asked for a 128-bit FNV-1a digest in 32 hex chars; a store names its bodies by the algorithm recorded in `store_meta.hash_algo`, and the live store names them in base32. A body written under the wrong name is a body no read ever finds. The hash comes from the handle now, and a requirement says so.

**`create_addressbook` refuses.** Declaring a collection is an owner write, and a collection no sync knows about is one no sync would carry, which is the reason `update_addressbook` and `delete_addressbook` already refused. All three `addressbook` writes now fail on pimdir, each naming its own reason.

**`create_card` reports a link id.** A queued create has no public `seq` until the owner applies it, so there is no store-assigned id to hand back.

Two things landed alongside. The backend did not compile against the current io-webdav either, so the CardDAV arm was brought up: `CarddavAddressbook` gained `supported_reports`, the card enumeration returns a `CarddavCardEnumOk` whose `truncated` flag now reaches the `propfind` report instead of being dropped, and `carddav report sync` grew a `--fallback` flag for a server implementing no `sync-collection`, the choice io-webdav deliberately leaves to the caller.

And the first run against a real store found a defect in shared code: `card list` never showed a `TEL` written above the card's `EMAIL`, because the preview chained its three reads with `else if` and a line that failed the EMAIL read never reached the TEL one. Posteo writes `TEL` first, so the column was empty for all 155 cards. Every backend was affected. The reads are independent now.

The [backends](../spec/backends.md) capability moved: two requirements added (the roles, and the store's own hash), three modified (the writes, the derivations, the addressbook limits), one removed (auto-source). The [pimdir test report](../spec/testing/pimdir-local.md) carries a second run, read-only against the live 155-card Posteo store. The write paths are unexercised there by design, and re-running the 2026-08-09 seeding harness against the queue model is the outstanding work on this backend.
