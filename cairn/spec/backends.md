---
cairn: spec
capability: backends
status: current
---

# Backends

Each backend is a `<Proto>Client` wrapper over the io-* `*Std` client (or, for the local stores, over the store handle), paired with a `src/<proto>/backend.rs` adapter implementing the unified operations and converting io-* results into the CLI's own shared types ([`Addressbook`](../../src/shared/addressbook/types.rs), [`Card`](../../src/shared/card/types.rs)). Cardamum owns these types: no aggregator library sits between it and the io-* crates, because the unified layer is a product decision with a single owner (the org's aggregator-retirement rule).

Every backend sits behind its own cargo feature (`carddav`, `jmap`, `msgraph`, `gpeople`, `vdir`, `pimdir`), so a build ships only the protocols it needs.

### Requirement: Unified operation set
The shared adapters SHALL cover, per backend: `list_addressbooks`, `create_addressbook`, `update_addressbook`, `delete_addressbook`, `list_cards`, `get_card`, `create_card`, `update_card` and `delete_card`. A backend that cannot model an operation SHALL fail with a clear message naming the limit rather than emulating it or silently succeeding.

### Requirement: Network backends
CardDAV, JMAP, Microsoft Graph and Google People SHALL each adapt their io-* high-level client. CardDAV reuses io-webdav's addressbook and card verbs over the resolved addressbook home set. JMAP reuses io-jmap's RFC 9610 `AddressBook` and `ContactCard` methods. Microsoft Graph maps contact folders to addressbooks over io-msgraph's `contact_folders` / `contacts` surface. Google People maps contact groups to addressbooks over io-people's `contactGroups` / `gpeople` surface.

### Requirement: A shared update never creates
The unified `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`, a filesystem write), the adapter SHALL establish that the target exists before writing: CardDAV reads the card's current version and guards the write with the ETag it returns, and vdir locates the item. An unknown id then fails on that read rather than creating anything. The wildcard `If-Match: *` SHALL NOT be used for this: RFC 9110 §13.1.1 defines it, but iCloud rejects it with `412` even for a resource that exists. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

### Requirement: Clearing an optional field removes it
A shared update SHALL distinguish "leave this field untouched" from "clear this field", and SHALL carry the difference all the way to the storage: on CardDAV a cleared property leaves as a `DAV:remove` instruction (RFC 4918 §9.2), never as an omitted `DAV:set`, on vdir a cleared property has its metadata file removed rather than left in place, and on JMAP a cleared description leaves as `null` in the `AddressBook/set` update. A backend that cannot express removal SHALL report that rather than accept the request and drop it. The adapter SHALL forward the diff as such, never reading the current state to merge unchanged fields by hand.

### Requirement: Unified outputs carry no backend details
The output of a unified command SHALL NOT carry a field that only one backend fills. A backend detail worth showing SHALL be logged by that backend's adapter or shown by its own namespace.

### Requirement: An unsupported precondition is refused, not dropped
A backend with no entity-tag concept SHALL refuse `--if-match` rather than accept and ignore it: its table declares `card.update.if-match` or `card.remove.if-match` at `none`, so the command fails with `capability-missing` before connecting, and the adapter keeps its own refusal. Silently dropping a guard the caller asked for is the one outcome that reads as protection and is none.

### Requirement: Card ids are backend-native and verbatim
A card's shared `id` SHALL be the backend's own identifier, used verbatim end to end: the last path segment of the resource href (CardDAV), the file stem (vdir), the `ContactCard` id (JMAP), the contact id (Graph), the person id (People), or the store-assigned public `seq` (pimdir). No adapter SHALL add or strip a file extension when addressing a resource. CardDAV names a new card `<uuid>.vcf` itself and passes the whole name through, since the server owns the resource name from then on. A CardDAV write SHALL report the id io-webdav answers, never the caller's string: `card create` and `carddav put` print the last segment of the `Location` the server names, else of the URL sent, percent-encoded as a listing shows it.

### Requirement: A listing is whole or fails
A listing SHALL never come back shorter than the backend holds without saying so: a card it could not read SHALL be an `errors` entry ("Each backend names the cards it could not read"), and a listing that cannot be read at all SHALL fail. A CardDAV listing whose answer is not one whole `multistatus` document (an HTML page, a cut body, an empty one) SHALL fail with io-webdav's error; a CardDAV listing the server truncated (a 507 row, RFC 6352 §8.6.1) SHALL add an entry with a `null` id and no code saying the listing is incomplete. A vdir listing SHALL leave out a card removed while it lists. A Graph listing SHALL follow its next page through io-msgraph's `page_get`, which refuses a link off the Graph origin, so the token never follows a link elsewhere.

### Requirement: Each backend names the cards it could not read
A backend SHALL report a card it could not read as an `errors` entry and SHALL never drop it silently nor fail the listing for it. A card named in `errors` SHALL NOT also list among the cards ("A listing names the items it could not read"):

- every backend: a card whose body does not parse as a vCard, or is not UTF-8, SHALL be a `card-unreadable` entry naming the card, no row listed for it;
- vdir: a card file that cannot be read SHALL be a `card-unreadable` entry naming the card, its error in `sources`; a card removed while it lists SHALL be left out with no entry;
- JMAP: a card that does not project SHALL be a `card-unreadable` entry, its id when it has one;
- pimdir: a body that cannot be read SHALL be a `card-unreadable` entry, no row listed for it, though its summary reads; a body the store does not hold yet SHALL be no entry, the card listed with its preview;
- CardDAV: a row the server answered without the card's data (a row refused whole, a refused `address-data` property, or none sent) SHALL be an entry naming the card, coded `permission-denied` when the row or a refused property carries a 403, else `card-unreadable`, each refused property and its status in `sources`; the entries print on the first page only, as the whole address book is read before the page is cut;
- CardDAV, Graph, People: a page or an answer that fails SHALL fail the listing whole.

### Requirement: Local storage backends
vdir and pimdir SHALL adapt io-vdir and io-pimdir. vdir stores each addressbook as an immediate subdirectory of `vdir.home-dir` and each card as a `.vcf` file inside it, byte-faithfully. pimdir is an offline cache a sync engine populates through io-pimdir, which holds both the store and the engine, and follows the cache requirements below.

### Requirement: A cache renames nothing it cannot push
The pimdir backend SHALL refuse every `addressbook update` field. Its collection row (id, display name, description, colour) is written by the sync from the server, and the backend stages item actions only, so any local edit of it is a change no sync would carry. A missing body SHALL fall back to the summary preview in a listing, as an unhydrated card does, rather than render as a blank row.

### Requirement: A pimdir addressbook is its collection id
The pimdir backend SHALL show and accept an address book as the store's collection id, verbatim: the collection `carddav/default` is the address book `carddav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one; the store is opaque to it, neither parsing nor validating an id (pimdir STORAGE §9.2) and modelling hierarchy through `parent` rather than through a separator. Shortening is therefore a guess at the producer's convention, and one that makes a single address book answer to two spellings.

An id SHALL name an address book or name nothing: one store holds the collections of every kind a sync caches, so the kind SHALL narrow them at the one seam both the listing and the id check read, never at the listing alone. The kind SHALL be compared on its media type, the part before any `;` parameter, trimmed, as io-pimdir compares it when it picks a collection's summaries. A kind-less collection counts, a sync having created one before kinds were declared.

An id naming no address book of the account SHALL be refused naming the ones it holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

### Requirement: pimdir is a cache, not a server
The pimdir backend SHALL treat the store as a possibly-partial cache. `get_card` on a card whose body is not local (`level < Full`, no stored object) SHALL report a clear "body not fetched" state, the cue to sync, rather than a data-loss error. The card still lists: `list_cards` SHALL project the stored contact summary (pimdir STORAGE Annex A) into a minimal preview vCard (`UID`, `FN`, `EMAIL`), the name escaped back into a content line, so a contact list reads correctly before a full sync, while `get_card` refuses outright so a preview can never be mistaken for the document of record.

### Requirement: A pimdir card listing streams the store
`list_cards` on pimdir SHALL walk the address book in keyset batches of the store's contacts order and hold one batch of rows and the page it returns, never the address book. The cards before the page SHALL be counted off their rows without reading a body, a body SHALL be read only for a card the page returns, and the walk SHALL stop once the page is full.

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`. The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them. The reader SHALL be built with the pending overlay, so an update or removal this process staged reads back before the store's owner applies it.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`enqueue_with_body`), which hashes it with BLAKE3, the one algorithm a store records in `store_meta.hash_algo` (pimdir STORAGE §5; a store recording another is refused as `store-hash-algo`), and SHALL NOT hash or write a body itself. A body named under another algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. A write carrying a body goes through the producer's own `enqueue_with_body`, which checks the action, then writes the body under its staging lock and appends the row pinning it in one transaction, so a refused write leaves no body behind; a `remove` goes through `enqueue`. Each runs the producer's gate once (pimdir STORAGE §15.6), cardamum running none of its own before it, and answers the partial supports the gate passed the action with, each logged as a warning. The action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A pimdir write reaches a synced address book or is refused
The pimdir backend SHALL refuse `create_card`, `update_card` and `delete_card`, before anything reaches the blob tree or the queue, on an address book no source syncs: one for which the store records neither a source checkpoint nor a binding (`collection_sources`). The owner applies a queued `add` there as whichever source drains first, binding it to a collection that source never lists, so the card would never be pushed; an `update` of a card no source binds applies to the store alone and a `remove` retains the card, neither reaching a server. A collection the device keeps for itself is the owner's to write (pimdir STORAGE §14.3), never a producer's.

### Requirement: A duplicate pimdir create is refused
`create_card` on pimdir SHALL refuse a card whose link id the address book already holds, as a live item or a pending create, appending nothing to the queue and writing nothing to the blob tree, with the code `item-duplicate`. The refusal SHALL be io-pimdir's (`PimdirProducer::enqueue_with_body`, checked in the transaction that would append the row, before the body is written), never a lookup of cardamum's own. The owner would park such an `add` at drain (pimdir STORAGE §15.3), so accepting it reports a success no sync carries out. With `-i`, the composer file is kept and named, as on any failed write.

### Requirement: A pimdir view is one account
The pimdir backend SHALL read one account of the store. With `pimdir.account` set it reads that one; unset, it SHALL derive the account from the store when its collections are grouped under one account or none, and SHALL refuse a store grouping several, naming them, rather than show their address books as one. A view mixing accounts is the Android and Linux apps' to offer, not a CLI's.

### Requirement: account check opens every role the backend takes
`account check` on pimdir SHALL open the store as a reader and list its address books, then open a producer and probe it (`PimdirProducer::probe`), which takes and releases the staging lock without waiting and the database's write lock, writing nothing into the store. A store whose staging lock or write lock cannot be taken fails the check rather than the first write; a collector holding the staging lock fails it at once with `store-busy`, and the check leaves nothing in objects/.

### Requirement: A pimdir card's version is its body hash
The pimdir backend SHALL report as a card's `etag`, in `card list` and `card read`, the store's hash of its body, the pending queue folded in, so a staged update moves it at once. A card whose body is not local SHALL report none, whether it was never fetched or its blob is missing from the store.

`update_card` and `delete_card` SHALL refuse, before anything is queued, a write whose `if_match` is not that version, surrounding double quotes ignored, with the code `precondition-failed` and an error starting `Precondition failed:`, naming the version found and the one expected. A card with no local body matches no version.

### Requirement: A queued pimdir edit names the body it replaced
`update_card` on pimdir SHALL state on the queued `update` the body the edit was made against (`if_match`, pimdir STORAGE §15.3): the version `--if-match` named, or the one `card update` read when given no vCard source, and SHALL state none only when the caller read no body. `delete_card` SHALL state on the queued `remove` the body the delete was decided over, the version `--if-match` named, and none without `--if-match`. The owner parks a row whose stated body the item no longer holds, so a server change a sync brings in between the staging and the drain is neither overwritten without a merge nor deleted unread. cardamum's write has already said it is queued, so the parked row shows in `pimdir status`, with the error that parked it.

#### Scenario: A delete of a card a sync changed since
- GIVEN a pimdir card deleted with `--if-match` naming the version read, the delete still queued
- WHEN a sync replaces the card's body, then drains the queue
- THEN the `remove` is parked, the card keeps the body the sync brought, and `pimdir status` lists the parked row

### Requirement: A pimdir write over a conflicted card is parked
`update_card` and `delete_card` on pimdir SHALL state no resolution (`resolves`, pimdir STORAGE §15.3) on the action they queue: cardamum shows no conflict and offers no command deciding one, and a producer states a resolution on a write deciding one alone. The owner therefore parks an `update` or a `remove` meeting a conflict, the card's or that of the binding it runs through (STORAGE §15.2), the card keeping its body and its diverging one, where a delete used to settle the conflict by its tombstone. cardamum queues no `move`, the third action a conflict parks since storage-draft-11 (pimdir SYNC §7), and no `copy`: its pimdir writes are `add`, `update` and `remove` alone. The write has already said it is queued, so the parked row shows in `pimdir status`, naming the conflict; the conflict is resolved by a client that shows it, or on the server.

#### Scenario: A delete of a conflicted card
- GIVEN a pimdir card two sources changed apart, its conflict undecided
- WHEN `cardamum card delete` names it and the sync engine drains the queue
- THEN the `remove` is parked, the card and its diverging body are kept, and `pimdir status` lists the parked row

### Requirement: pimdir derivations match the sync engine
The link id, summary and sort key of a card a pimdir write stages SHALL be io-pimdir's own derivations (`io_pimdir::summary`, pimdir STORAGE Annex A), and the backend SHALL carry none of its own. A queued `add` or `update` names the body alone: the store's owner derives the summary and the sort key from it when it applies the action, so a card Cardamum stages summarizes exactly as the same card arriving through a sync. The link id `create_card` reports is the same derivation run on the same bytes: the bare `UID`, or `esc:` and the `UID` when it begins with a prefix the store reserves for the keys it derives (`alt:`, `hash:`, `dup:`, `part:`, `file:`, `esc:`, pimdir STORAGE §9), or `hash:` over the body when the card states none, or one that is empty or of spaces and tabs alone.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir STORAGE §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

### Requirement: A store refusal carries its code
The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open, read the queue of, or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `lock-replaced`, `unsupported`, `item-duplicate` and `action-malformed`; an SQL, I/O or JSON failure carries none, and so does a code no cardamum path raises (`flag-invalid` among them: cardamum stages no flag, its `add` carrying the empty set and its `update` and `remove` none). `action-malformed` reaches cardamum on `pimdir status`, whose count of queued creates decodes every pending row of the address book, when another program queued a payload its schema refuses: the owner's drain parks it, and the status reads again. `lock-replaced` reaches cardamum on a write, the producer checking before its enqueue that the staging lock file it holds is still the one at its path (pimdir STORAGE §8): nothing is queued. The message is io-pimdir's, but for `store-version` (see "A pimdir store of another version is reinstalled"), and SHALL NOT advise deleting the store: a resync restores neither the queue nor what the store keeps alone.

### Requirement: A server's refusal carries its code
A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, in the failure object, an `errors` entry and an `account check` row alike. One reader per backend codes its typed errors, walking the error chain after any coded error, so the typed errors stay in the chain the backends return; the failure object is coded once the command failed, its messages and causes printing as before.

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Graph, People); a CardDAV 403 whose body names no precondition, or `need-privileges` (RFC 3744 §7.1.1); a CardDAV listing, `report query` or `report multiget` row, or its refused `address-data`, answered 403; a JMAP `forbidden` |
| `quota-exceeded` | an HTTP 507; a CardDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded`; a JMAP `overQuota` |
| `rate-limited` | an HTTP 429, or a People 403 naming a rate limit or a quota, or a JMAP `rateLimit`; cardamum retries nothing, so the first one met |
| `card-not-found` | a card request answered 404 or 410 (CardDAV, People), a `carddav report multiget` row included, 404 or 400 `ErrorInvalidIdMalformed` (Graph), `notFound` (JMAP ContactCard); a vdir card file or a pimdir card that does not exist, or a pimdir card id that is no number |
| `addressbook-not-found` | an address book named that does not exist (vdir, pimdir); a JMAP AddressBook `notFound` |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CardDAV 403 naming another precondition (`valid-address-data`, `no-uid-conflict`) is a card the server would not take, no matter of rights, and SHALL carry none; a refused `valid-address-data` keeps the server's answer as the cause of cardamum's hint.

#### Scenario: A read-only address book
- GIVEN a Graph contact folder the account may read but not write
- WHEN `cardamum --json card delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

### Requirement: A pimdir store of another version is reinstalled
The pimdir backend SHALL open a store of the format version the io-pimdir it builds against reads (`io_pimdir::sql::FORMAT_VERSION`, `storage-draft-11` at the io-pimdir commit Cargo.toml pins) and no other, earlier or later, refusing it as `store-version` and reading nothing in it (pimdir STORAGE §4.2, §6). Nothing migrates a draft store. The refusal's message is cardamum's, not io-pimdir's: it names the version the store records, or that it records none, and the one this cardamum reads, and says that a store an earlier neverest wrote is reinstalled (that neverest's sync run until nothing is queued, neverest upgraded, the store moved aside, `neverest init` and a sync), while a store a later neverest wrote asks for a cardamum upgrade. It SHALL NOT advise deleting the store, nor syncing a new one before the queue is drained: the queued writes live in that store alone.

#### Scenario: A store an earlier neverest wrote
- GIVEN a pimdir store whose `store_meta.format_version` is `storage-draft-10`
- WHEN any cardamum command opens it
- THEN it fails with the code `store-version` and a message naming `storage-draft-10` and the neverest reinstall, its queue drained first

### Requirement: A pimdir store others can read is refused
The pimdir backend SHALL carry `store-exposed` when io-pimdir refuses a store whose directory or pimdir.db another user can read or write, or a link where a lock file belongs (pimdir STORAGE §18), before anything in it is opened. The message is io-pimdir's: restrict the store (`chmod 700` its directory, `600` its pimdir.db), never remove it.

### Requirement: A UID is not an address
The pimdir backend SHALL NOT assume an item's link id is the `UID` its body carries, nor that a `UID` identifies at most one card in a collection. A store may hold two cards of one address book sharing a `UID`, keyed apart by the store (pimdir STORAGE §9), and both SHALL list, read and act as ordinary cards, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one card. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 6352 §5.1 requires the `UID` to be unique in the collection and servers do not always enforce it, most often after a repeated import. The store now keeps both copies rather than one, and a backend resolving an identity to whichever row came first would hide a card the server holds.

#### Scenario: A duplicated card lists twice
- GIVEN an address book whose store holds two items whose bodies carry one `UID`
- WHEN the backend lists it
- THEN both appear, each with its own public id, and neither is marked

### Requirement: pimdir store path is shell-expanded
The pimdir backend SHALL expand `~` and environment variables on `pimdir.root` before opening the store and its blob reader. Opening the raw path would look for a store at a literal `./~/…` relative to the cwd, and report that one is missing while the real store sits untouched.

### Requirement: pimdir addressbook limits are explicit
Every `addressbook` write SHALL fail on pimdir, each naming its reason. `create_addressbook` refuses because declaring a collection is an owner write and this backend is a producer, and because a collection no sync knows about is one no sync would carry. `update_addressbook` refuses because the collection row (id, display name, description, colour) is what a sync writes from the server, and io-pimdir's `rename_collection` renames the identifier rather than the display name. `delete_addressbook` refuses, because io-pimdir exposes no collection removal, with a message naming the alternative: delete it on the server and sync. It SHALL NOT advise removing the store, which loses the queued writes a resync never restores.

### Requirement: Backend selection is local-first
The unified commands run over an `AddressbookClient` owning one `BackendClient` variant per compiled-in backend. The global `--backend` flag picks it: `auto` (the default) SHALL take the first configured-and-allowed backend in priority order, which puts the local stores (vdir, then pimdir) before the network ones so an offline store wins over a round-trip. A named value pins that backend and bails when the account has no matching config block. The order SHALL walk every block the account declares, the backends the build leaves out included, so the backend `auto` resolves to is the same in every build. The protocol-specific commands build their own client and ignore `--backend` entirely.

### Requirement: A backend left out of the build is named
When the backend `auto` resolves to is one the build leaves out, the unified commands and `account capability` SHALL fail coded `backend-not-built`, the message naming the cargo feature ("This build leaves out the `gpeople` backend, rebuild with `--features gpeople`"), rather than take the next declared backend or report none configured. `--backend` naming a backend the build leaves out SHALL be refused as a command line (`usage-invalid`, exit 2), with the same message. Under `auto`, `account check` SHALL report each declared backend the build leaves out as a failed row coded `backend-not-built`, at its place in the resolution order. `account list` SHALL name under `--json` the declared backends the build carries in `backends`, each a value `--backend` takes, and those it leaves out in `notBuilt`, both in the resolution order; the text SHALL print both in one cell, in that order, each left-out one marked `(not built)`.

#### Scenario: account list keeps the two apart
- GIVEN a build with `vdir` and without `gpeople`, and an account declaring both
- WHEN `cardamum --json account list` runs
- THEN the account reads `"backends":["vdir"]` and `"notBuilt":["gpeople"]`, and the text cell `vdir,gpeople (not built)`

#### Scenario: An account over a backend left out
- GIVEN a build without `gpeople` and an account declaring `gpeople` alone
- WHEN `cardamum --json card list -k book` runs
- THEN it exits 1, its failure object coded `backend-not-built`, its error naming `--features gpeople`

#### Scenario: auto keeps its choice across builds
- GIVEN a build without `vdir` and an account declaring `vdir` and `carddav`
- WHEN `cardamum --json account capability` runs
- THEN it fails coded `backend-not-built`, where `--backend carddav` resolves to CardDAV

### Requirement: CardDAV home-set resolution routes
The `[carddav]` block SHALL resolve the addressbook home set through one of two routes. `home` short-circuits all discovery and is used as the home set directly. `server` connects to the given context root, then walks current-user-principal (RFC 5397) and addressbook-home-set (RFC 6352). A block naming neither SHALL fail naming both.

### Requirement: account check proves a CardDAV home
`account check` on a `[carddav]` block naming `home` SHALL send the home one authenticated `PROPFIND` of depth 0, asking for `DAV:resourcetype`, once connected, and SHALL fail the `carddav` row when it is refused, with the message and the code the `server` route's discovery fails with on the same answer: io-webdav's `WebDAV server returned HTTP <status>`, a 401 carrying no code and a 403 `permission-denied` ("A server's refusal carries its code"). The `home` route connects without a request, so nothing else in the check proves the credential. The request SHALL be the check's alone, the other commands sending none: the first request each sends is authenticated, so a refused credential fails it, and the `home` route exists to save the discovery round trips. The `server` route sends none either, its walk proving the credential.

#### Scenario: A made-up token against a CardDAV home
- GIVEN an account whose `carddav.home` names a server answering its token 401
- WHEN `cardamum --json account check` runs
- THEN it exits 3, the `carddav` row `ok: false`, `code` `null` and its `error` naming HTTP 401, as with `carddav.server` naming the same server

### Requirement: CardDAV probes a bare origin before walking
Because a bare origin is not necessarily the context root (Fastmail serves contacts under `/dav/` and 404s everything else), the `server` route SHALL probe `.well-known/carddav` and follow its redirect before the principal walk whenever the server names no path.

### Requirement: The connection is opened by the call that needs it
`AddressbookClient` SHALL select its backend from the configuration without connecting, and open the connection on the first call that needs one. A command that never reaches the network therefore never opens a socket, and a command that runs a composer holds none open while the editor is up.

That last case is why the rule exists rather than being an optimization. A server closes an idle connection, and an editor session lasts minutes: a write landing after one reads the end of a socket whose far side is gone, and reports `unexpected end of file` for a card that is perfectly good. `card create -i` connects for the first time when it creates, after the editor has exited.

A command that must read before the editor SHALL drop that connection before spawning the composer, so the write after it opens a fresh one. `card update -i` is the only such command: it reads the card to seed the editor with it.
