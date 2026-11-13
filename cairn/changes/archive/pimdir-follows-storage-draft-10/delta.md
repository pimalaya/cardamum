---
cairn: delta
change: pimdir-follows-storage-draft-10
---

# Delta

Each requirement is of the capability named in parentheses under its heading.

## ADDED Requirements

(backends)

### Requirement: A pimdir write over a conflicted card is parked
`update_card` and `delete_card` on pimdir SHALL state no resolution (`resolves`, pimdir STORAGE §15.3) on the action they queue: cardamum shows no conflict and offers no command deciding one, and a producer states a resolution on a write deciding one alone. The owner therefore parks an `update` or a `remove` meeting a conflict, the card's or that of the binding it runs through (STORAGE §15.2), the card keeping its body and its diverging one, where a delete used to settle the conflict by its tombstone. The write has already said it is queued, so the parked row shows in `pimdir status`, naming the conflict; the conflict is resolved by a client that shows it, or on the server.

#### Scenario: A delete of a conflicted card
- GIVEN a pimdir card two sources changed apart, its conflict undecided
- WHEN `cardamum card delete` names it and the sync engine drains the queue
- THEN the `remove` is parked, the card and its diverging body are kept, and `pimdir status` lists the parked row

## MODIFIED Requirements

(backends)

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`enqueue_with_body`), which hashes it with BLAKE3, the one algorithm a store records in `store_meta.hash_algo` (pimdir STORAGE §5; a store recording another is refused as `store-hash-algo`), and SHALL NOT hash or write a body itself. A body named under another algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. A write carrying a body goes through the producer's own `enqueue_with_body`, which checks the action, then writes the body under its staging lock and appends the row pinning it in one transaction, so a refused write leaves no body behind; a `remove` goes through `enqueue`. Each runs the producer's gate once (pimdir STORAGE §15.6), cardamum running none of its own before it, and answers the partial supports the gate passed the action with, each logged as a warning. The action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A duplicate pimdir create is refused
`create_card` on pimdir SHALL refuse a card whose link id the address book already holds, as a live item or a pending create, appending nothing to the queue and writing nothing to the blob tree, with the code `item-duplicate`. The refusal SHALL be io-pimdir's (`PimdirProducer::enqueue_with_body`, checked in the transaction that would append the row, before the body is written), never a lookup of cardamum's own. The owner would park such an `add` at drain (pimdir STORAGE §15.3), so accepting it reports a success no sync carries out. With `-i`, the composer file is kept and named, as on any failed write.

### Requirement: A queued pimdir edit names the body it replaced
`update_card` on pimdir SHALL state on the queued `update` the body the edit was made against (`if_match`, pimdir STORAGE §15.3): the version `--if-match` named, or the one `card update` read when given no vCard source, and SHALL state none only when the caller read no body. `delete_card` SHALL state on the queued `remove` the body the delete was decided over, the version `--if-match` named, and none without `--if-match`. The owner parks a row whose stated body the item no longer holds, so a server change a sync brings in between the staging and the drain is neither overwritten without a merge nor deleted unread. cardamum's write has already said it is queued, so the parked row shows in `pimdir status`, with the error that parked it.

#### Scenario: A delete of a card a sync changed since
- GIVEN a pimdir card deleted with `--if-match` naming the version read, the delete still queued
- WHEN a sync replaces the card's body, then drains the queue
- THEN the `remove` is parked, the card keeps the body the sync brought, and `pimdir status` lists the parked row

### Requirement: pimdir derivations match the sync engine
The link id, summary and sort key of a card a pimdir write stages SHALL be io-pimdir's own derivations (`io_pimdir::summary`, pimdir STORAGE Annex A), and the backend SHALL carry none of its own. A queued `add` or `update` names the body alone: the store's owner derives the summary and the sort key from it when it applies the action, so a card Cardamum stages summarizes exactly as the same card arriving through a sync. The link id `create_card` reports is the same derivation run on the same bytes: the bare `UID`, or `esc:` and the `UID` when it begins with a prefix the store reserves for the keys it derives (`alt:`, `hash:`, `dup:`, `part:`, `file:`, `esc:`, pimdir STORAGE §9), or `hash:` over the body when the card states none, or one that is empty or of spaces and tabs alone.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir STORAGE §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

### Requirement: A store refusal carries its code
The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open, read the queue of, or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `lock-replaced`, `unsupported`, `item-duplicate` and `action-malformed`; an SQL, I/O or JSON failure carries none, and so does a code no cardamum path raises (`flag-invalid` among them: cardamum stages no flag, its `add` carrying the empty set and its `update` and `remove` none). `action-malformed` reaches cardamum on `pimdir status`, whose count of queued creates decodes every pending row of the address book, when another program queued a payload its schema refuses: the owner's drain parks it, and the status reads again. `lock-replaced` reaches cardamum on a write, the producer checking before its enqueue that the staging lock file it holds is still the one at its path (pimdir STORAGE §8): nothing is queued. The message is io-pimdir's, but for `store-version` (see "A pimdir store of another version is reinstalled"), and SHALL NOT advise deleting the store: a resync restores neither the queue nor what the store keeps alone.

### Requirement: A pimdir store of another version is reinstalled
The pimdir backend SHALL open a store of the format version the io-pimdir it builds against reads (`io_pimdir::sql::FORMAT_VERSION`, `storage-draft-10` at io-pimdir 0.11) and no other, earlier or later, refusing it as `store-version` and reading nothing in it (pimdir STORAGE §4.2, §6). Nothing migrates a draft store. The refusal's message is cardamum's, not io-pimdir's: it names the version the store records, or that it records none, and the one this cardamum reads, and says that a store an earlier neverest wrote is reinstalled (that neverest's sync run until nothing is queued, neverest upgraded, the store moved aside, `neverest init` and a sync), while a store a later neverest wrote asks for a cardamum upgrade. It SHALL NOT advise deleting the store, nor syncing a new one before the queue is drained: the queued writes live in that store alone.

#### Scenario: A store an earlier neverest wrote
- GIVEN a pimdir store whose `store_meta.format_version` is `storage-draft-09`
- WHEN any cardamum command opens it
- THEN it fails with the code `store-version` and a message naming `storage-draft-09` and the neverest reinstall, its queue drained first

(commands)

### Requirement: A pimdir card search says what it did not cover
The output of `pimdir search` and `pimdir count` SHALL carry the coverage of the answer (SEARCH §8.2): `bodiless`, the cards in scope whose body is not local, matched on their summary alone; `missing`, those whose body the index could not read, and `unparseable`, those whose body no parser accepted, both counted only when a term of the query reads card bodies (free text, `has:phone`, `has:address`, `has:birthday`, `has:photo`, `category:`), a query of summary terms alone counting none; `unindexed`, those whose last change the index has not folded in; `behind`, how many changes the store drew above the index, store-wide; `filling`, whether a first round is still listing an address book in scope; and `complete`, io-pimdir's own verdict (`PimdirSearchCoverage::is_complete`, the lag included), true only when none of these holds. The keys are flat and in camelCase, cardamum's own shape over the per-kind counts of SEARCH §8.2, every query being scoped to cards. The plain output SHALL print one line under its table naming what did not hold, and nothing when the answer is complete.

#### Scenario: A card the index could not parse
- GIVEN an indexed pimdir address book holding a card whose body no parser accepted
- WHEN `cardamum pimdir count has:phone` runs
- THEN the coverage reports `unparseable` 1 and `complete` false, and the plain output names the card that does not parse; `cardamum pimdir count email:@example.org` reports `unparseable` 0 and, nothing else missing, `complete` true

#### Scenario: An index behind the store
- GIVEN a card updated in the store since the owner's last refresh
- WHEN `cardamum pimdir count kind:contact` runs
- THEN the coverage reports `behind` above 0, `complete` false, and the plain output says the sync engine refreshes the index

In "A JSON error carries a stable code", the table gains the row `lock-replaced`; in "pimdir searches cards through the store's index", the query language is that of search-draft-10 §8.

## REMOVED Requirements

None.
