---
cairn: change
change: pimdir-release-alignment
---

# Delta

## ADDED Requirements

### Requirement: A pimdir write reaches a synced address book or is refused
The pimdir backend SHALL refuse `create_card`, `update_card` and `delete_card`, before anything reaches the blob tree or the queue, on an address book no source syncs: one for which the store records neither a source checkpoint nor a binding (`collection_sources`). The owner applies a queued action as a source syncing its collection, and with none it binds whichever source drains first to a collection that source never lists, so the card would never be pushed. A collection the device keeps for itself is the owner's to write (pimdir STORAGE §14.3), never a producer's.

### Requirement: A pimdir view is one account
The pimdir backend SHALL read one account of the store. With `pimdir.account` set it reads that one; unset, it SHALL derive the account from the store when its collections are grouped under one account or none, and SHALL refuse a store grouping several, naming them, rather than show their address books as one. A view mixing accounts is the Android and Linux apps' to offer, not a CLI's.

### Requirement: A queued write says it is queued
A shared write SHALL say whether it reached the server or was queued for a sync. On pimdir, `card create`, `card update` and `card delete` SHALL say the write is queued and that the next sync carries it, never that the card was created, updated or deleted. `card create` and `card update` SHALL carry a `queued` boolean in `--json`, `false` on every backend whose writes reach the server as they are made, which keeps the field least-common-denominator rather than a pimdir detail.

A queued creation SHALL get no placeholder id (`q<id>`, `0`, `""`): `card create` reports the card's link id, which `card read` does not accept until a sync applies the create and the card lists under its public id.

### Requirement: account check opens every role the backend takes
`account check` on pimdir SHALL open the store as a reader and list its address books, then open and drop a producer, so a store whose staging lock cannot be taken fails the check rather than the first write.

## MODIFIED Requirements

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`. The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them. The reader SHALL be built with the pending overlay, so an update or removal this process staged reads back before the store's owner applies it.

### Requirement: A pimdir addressbook is its collection id
The pimdir backend SHALL show and accept an address book as the store's collection id, verbatim: the collection `carddav/default` is the address book `carddav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one; the store is opaque to it, neither parsing nor validating an id (pimdir STORAGE §9.2) and modelling hierarchy through `parent` rather than through a separator. Shortening is therefore a guess at the producer's convention, and one that makes a single address book answer to two spellings.

An id SHALL name an address book or name nothing: one store holds the collections of every kind a sync caches, so the kind SHALL narrow them at the one seam both the listing and the id check read, never at the listing alone. The kind SHALL be compared on its media type, the part before any `;` parameter, trimmed, as io-pimdir compares it when it picks a collection's summaries. A kind-less collection counts, a sync having created one before kinds were declared.

An id naming no address book of the account SHALL be refused naming the ones it holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. The body reaches the blob tree through the blob writer before the row that pins it is appended, and the action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A pimdir card's version is its body hash
The pimdir backend SHALL report as a card's `etag`, in `card list` and `card read`, the store's hash of its body, the pending queue folded in, so a staged update moves it at once. A card whose body is not local SHALL report none, whether it was never fetched or its blob is missing from the store.

`update_card` and `delete_card` SHALL refuse, before anything is queued, a write whose `if_match` is not that version, surrounding double quotes ignored, with the code `precondition-failed` and an error starting `Precondition failed:`, naming the version found and the one expected. A card with no local body matches no version.

### Requirement: pimdir derivations match the sync engine
The link id, summary and sort key of a card a pimdir write stages SHALL be io-pimdir's own derivations (`io_pimdir::summary`, pimdir STORAGE Annex A), and the backend SHALL carry none of its own. A queued `add` or `update` names the body alone: the store's owner derives the summary and the sort key from it when it applies the action, so a card Cardamum stages summarizes exactly as the same card arriving through a sync. The link id `create_card` reports is the same derivation run on the same bytes: the bare `UID`, with nothing prepended, or `hash:` over the body when the card states none.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir STORAGE §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

### Requirement: A store from an earlier draft is refused
The pimdir backend SHALL open a store as the current format defines it and SHALL NOT migrate one an earlier draft wrote. io-pimdir refuses such a store naming the table it lacks, and the backend reports that as the open failure, the fix being to delete the store and let the sync recreate it, the draft offering no migration.

### Requirement: A UID is not an address
The pimdir backend SHALL NOT assume an item's link id is the `UID` its body carries, nor that a `UID` identifies at most one card in a collection. A store may hold two cards of one address book sharing a `UID`, keyed apart by the store (pimdir STORAGE §9), and both SHALL list, read and act as ordinary cards, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one card. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 6352 §5.1 requires the `UID` to be unique in the collection and servers do not always enforce it, most often after a repeated import. The store now keeps both copies rather than one, and a backend resolving an identity to whichever row came first would hide a card the server holds.

#### Scenario: A duplicated card lists twice
- GIVEN an address book whose store holds two items whose bodies carry one `UID`
- WHEN the backend lists it
- THEN both appear, each with its own public id, and neither is marked

## REMOVED Requirements
