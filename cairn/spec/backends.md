---
cairn: spec
capability: backends
status: current
---

# Backends

Each backend is a `<Proto>Client` wrapper over the io-* `*Std` client (or, for the local stores, over the store handle), paired with a `src/<proto>/backend.rs` adapter implementing the shared operations and converting io-* results into the CLI's own shared types ([`Addressbook`](../../src/shared/addressbook/types.rs), [`Card`](../../src/shared/card/types.rs)). Cardamum owns these types: no aggregator library sits between it and the io-* crates, because the least-common-denominator layer is a product decision with a single owner (the org's aggregator-retirement rule).

Every backend sits behind its own cargo feature (`carddav`, `jmap`, `msgraph`, `people`, `vdir`, `pimdir`), so a build ships only the protocols it needs.

### Requirement: Shared operation set
The shared adapters SHALL cover, per backend: `list_addressbooks`, `create_addressbook`, `update_addressbook`, `delete_addressbook`, `list_cards`, `get_card`, `create_card`, `update_card` and `delete_card`. A backend that cannot model an operation SHALL fail with a clear message naming the limit rather than emulating it or silently succeeding.

### Requirement: Network backends
CardDAV, JMAP, Microsoft Graph and Google People SHALL each adapt their io-* high-level client. CardDAV reuses io-webdav's addressbook and card verbs over the resolved addressbook home set. JMAP reuses io-jmap's RFC 9610 `AddressBook` and `ContactCard` methods. Microsoft Graph maps contact folders to addressbooks over io-msgraph's `contact_folders` / `contacts` surface. Google People maps contact groups to addressbooks over io-people's `contactGroups` / `people` surface.

### Requirement: A shared update never creates
The shared `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`, a filesystem write), the adapter SHALL establish that the target exists before writing: CardDAV reads the card's current version and guards the write with the ETag it returns, and vdir locates the item. An unknown id then fails on that read rather than creating anything. The wildcard `If-Match: *` SHALL NOT be used for this: RFC 9110 §13.1.1 defines it, but iCloud rejects it with `412` even for a resource that exists. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

### Requirement: Clearing an optional field removes it
A shared update SHALL distinguish "leave this field untouched" from "clear this field", and SHALL carry the difference all the way to the storage: on CardDAV a cleared property leaves as a `DAV:remove` instruction (RFC 4918 §9.2), never as an omitted `DAV:set`, and on vdir a cleared property has its metadata file removed rather than left in place. A backend that cannot express removal SHALL report that rather than accept the request and drop it. The adapter SHALL forward the diff as such, never reading the current state to merge unchanged fields by hand.

### Requirement: An unsupported precondition is refused, not dropped
A backend with no entity-tag concept SHALL refuse `--if-match` rather than accept and ignore it, as msgraph, jmap and vdir do. Silently dropping a guard the caller asked for is the one outcome that reads as protection and is none.

### Requirement: Card ids are backend-native and verbatim
A card's shared `id` SHALL be the backend's own identifier, used verbatim end to end: the last path segment of the resource href (CardDAV), the file stem (vdir), the `ContactCard` id (JMAP), the contact id (Graph), the person id (People), or the store-assigned public `seq` (pimdir). No adapter SHALL add or strip a file extension when addressing a resource. CardDAV names a new card `<uuid>.vcf` itself and passes the whole name through, since the server owns the resource name from then on.

### Requirement: Local storage backends
vdir and pimdir SHALL adapt io-vdir and io-pimdir. vdir stores each addressbook as an immediate subdirectory of `vdir.home-dir` and each card as a `.vcf` file inside it, byte-faithfully. pimdir is an offline cache the sync engine (io-replica plus io-pimdir) populates, and follows the cache requirements below.

### Requirement: A cache renames nothing it cannot push
The pimdir backend SHALL refuse every `addressbook update` field. Its collection row (id, display name, description, colour) is written by the sync from the server, and the backend stages item actions only, so any local edit of it is a change no sync would carry. A missing body SHALL fall back to the summary preview in a listing, as an unhydrated card does, rather than render as a blank row.

### Requirement: A pimdir addressbook is its collection id
The pimdir backend SHALL show and accept an address book as the store's collection id, verbatim: the collection `carddav/default` is the address book `carddav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one; the store is opaque to it, neither parsing nor validating an id (pimdir SPEC 9.2) and modelling hierarchy through `parent` rather than through a separator. Shortening is therefore a guess at the producer's convention, and one that makes a single address book answer to two spellings.

An id SHALL name an address book or name nothing: one store holds the collections of every kind a sync caches, so the kind SHALL narrow them at the one seam both the listing and the id check read, never at the listing alone. A kind-less collection counts, a sync having created one before kinds were declared.

An id naming no address book of the account SHALL be refused naming the ones it holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

### Requirement: pimdir is a cache, not a server
The pimdir backend SHALL treat the store as a possibly-partial cache. `get_card` on a card whose body is not local (`level < Full`, no stored object) SHALL report a clear "body not fetched" state, the cue to sync, rather than a data-loss error. The card still lists: `list_cards` SHALL project the stored `v: 1` summary into a minimal preview vCard (`UID`, `FN`, `EMAIL`) so a contact list reads correctly before a full sync, while `get_card` refuses outright so a preview can never be mistaken for the document of record.

### Requirement: pimdir takes the reader and producer roles, never the owner
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL NOT open a `PimdirStore`. The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it would both lock a sync out for the length of a listing and put every destructive verb behind a frontend that never calls them. The reader SHALL be built with the pending overlay, so an action this process staged reads back before the store's owner applies it.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL name a body it writes with the hash the store records in `store_meta.hash_algo`, read through the handle it holds, and SHALL NOT compute a digest of its own choosing. A body named under the wrong algorithm is a body no read ever finds.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir SPEC §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. The body reaches the blob tree through the blob writer before the row that pins it is appended, and the action addresses the item by the public `seq` that is already the card's shared id. `update_card` ignores `--if-match`, because the engine reconciles the applied edit against the base body it recorded at sync time, which is stronger than an ETag precondition a local store cannot check. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

### Requirement: pimdir derivations match the sync engine
The link id and the `v: 1` summary a pimdir write records SHALL be derived by `io_pimdir::conventions::card`, the format's own derivations, so a card Cardamum stages links and summarizes exactly as the same card arriving through a sync. The link id is the bare `UID`, with nothing prepended. A queued action carries no sort key: the format leaves the key to the sync that pushes the create, and a producer deriving one would order an item the connector is about to reorder.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir SPEC §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

### Requirement: A UID is not an address
The pimdir backend SHALL NOT assume an item's link id is the `UID` its body carries, nor that a `UID` identifies at most one card in a collection. A store may hold two cards of one address book sharing a `UID`, keyed apart by the store (pimdir SPEC §9), and both SHALL list, read and act as ordinary cards, addressed by their own public `seq`.

What stays unique is the key and the public id: `(collection, link_id)` still names one item and `seq` still names one card. What ends is the link id being derivable from the body, so a read that re-derives a `UID` in order to address a row is addressing an unknown number of them.

RFC 6352 §5.1 requires the `UID` to be unique in the collection and servers do not always enforce it, most often after a repeated import. The store now keeps both copies rather than one, and a backend resolving an identity to whichever row came first would hide a card the server holds.

#### Scenario: A duplicated card lists twice
- GIVEN an address book whose store holds two items whose bodies carry one `UID`
- WHEN the backend lists it
- THEN both appear, each with its own public id, and neither is marked

### Requirement: pimdir store path is shell-expanded
The pimdir backend SHALL expand `~` and environment variables on `pimdir.root` before opening the store and its blob reader. Opening the raw path would look for a store at a literal `./~/…` relative to the cwd, and report that one is missing while the real store sits untouched.

### Requirement: pimdir addressbook limits are explicit
Every `addressbook` write SHALL fail on pimdir, each naming its reason. `create_addressbook` refuses because declaring a collection is an owner write and this backend is a producer, and because a collection no sync knows about is one no sync would carry. `update_addressbook` refuses because the collection row (id, display name, description, colour) is what a sync writes from the server, and io-pimdir's `rename_collection` renames the identifier rather than the display name. `delete_addressbook` refuses with a message naming the alternative (delete on the server and sync, or remove the store directory), because io-pimdir exposes no collection removal.

### Requirement: Backend selection is local-first
The shared commands run over an `AddressbookClient` owning one `BackendClient` variant per compiled-in backend. The global `--backend` flag picks it: `auto` (the default) SHALL take the first configured-and-allowed backend in priority order, which puts the local stores (vdir, then pimdir) before the network ones so an offline store wins over a round-trip. A named value pins that backend and bails when the account has no matching config block. The protocol-specific commands build their own client and ignore `--backend` entirely.

### Requirement: CardDAV home-set resolution routes
The `[carddav]` block SHALL resolve the addressbook home set through one of three routes, in decreasing order of magic. `home` short-circuits all discovery and is used as the home set directly. `server` connects to the given context root, then walks current-user-principal (RFC 5397) and addressbook-home-set (RFC 6352). `discover` resolves a bare domain to a context root before that same walk: PACC first, then RFC 6764 (SRV record, its TXT `path`, then `.well-known`) through io-pim-discovery, with Google domains using an authenticated `.well-known` probe.

### Requirement: CardDAV probes a bare origin before walking
Because PACC and RFC 6764 can hand back a bare origin rather than the real context root (Fastmail serves contacts under `/dav/` and 404s everything else), the `server` and `discover` routes SHALL probe `.well-known/carddav` and follow its redirect before the principal walk whenever the resolved path is `/`.

### Requirement: The connection is opened by the call that needs it
`AddressbookClient` SHALL select its backend from the configuration without connecting, and open the connection on the first call that needs one. A command that never reaches the network therefore never opens a socket, and a command that runs a composer holds none open while the editor is up.

That last case is why the rule exists rather than being an optimization. A server closes an idle connection, and an editor session lasts minutes: a write landing after one reads the end of a socket whose far side is gone, and reports `unexpected end of file` for a card that is perfectly good. `card create -i` connects for the first time when it creates, after the editor and after the menu.

A command that must read before the editor SHALL drop that connection before spawning the composer, so the write after it opens a fresh one. `card update -i` is the only such command: it reads the card to seed the editor with it.
