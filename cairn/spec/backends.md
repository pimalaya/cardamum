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
The shared `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`), the adapter SHALL read the target's current version first and guard the write with the ETag it returns, so that an unknown id fails on the read as "not found". The wildcard `If-Match: *` SHALL NOT be used for this: RFC 9110 §13.1.1 defines it, but iCloud rejects it with `412` even for a resource that exists. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

### Requirement: Clearing an optional field removes it
A shared update SHALL distinguish "leave this field untouched" from "clear this field", and SHALL carry the difference all the way to the wire: on CardDAV a cleared property leaves as a `DAV:remove` instruction (RFC 4918 §9.2), never as an omitted `DAV:set`. A backend that cannot express removal SHALL report that rather than accept the request and drop it. The adapter SHALL forward the diff as such, never reading the current state to merge unchanged fields by hand.

### Requirement: Card ids are backend-native and verbatim
A card's shared `id` SHALL be the backend's own identifier, used verbatim end to end: the last path segment of the resource href (CardDAV), the file stem (vdir), the `ContactCard` id (JMAP), the contact id (Graph), the person id (People), or the store-assigned public `seq` (pimdir). No adapter SHALL add or strip a file extension when addressing a resource. CardDAV names a new card `<uuid>.vcf` itself and passes the whole name through, since the server owns the resource name from then on.

### Requirement: Local storage backends
vdir and pimdir SHALL adapt io-vdir and io-pimdir. vdir stores each addressbook as an immediate subdirectory of `vdir.home-dir` and each card as a `.vcf` file inside it, byte-faithfully. pimdir is an offline cache the sync engine (io-replica plus io-pimdir) populates, and follows the cache requirements below.

### Requirement: A cache renames nothing it cannot push
The pimdir backend SHALL refuse every `addressbook update` field. Its collection row (id, display name, description, colour) is written by the sync from the server, and the backend stages item mutations only, so any local edit of it is a change no sync would carry. A missing body SHALL fall back to the summary preview in a listing, as an unhydrated card does, rather than render as a blank row.

### Requirement: pimdir is a cache, not a server
The pimdir backend SHALL treat the store as a possibly-partial cache. `get_card` on a card whose body is not local (`level < Full`, no stored object) SHALL report a clear "body not fetched" state, the cue to sync, rather than a data-loss error. The card still lists: `list_cards` SHALL project the stored `v: 1` summary into a minimal preview vCard (`UID`, `FN`, `EMAIL`) so a contact list reads correctly before a full sync, while `get_card` refuses outright so a preview can never be mistaken for the document of record.

### Requirement: pimdir writes are staged mutations
A pimdir write SHALL stage an io-replica `ReplicaMutation` through the store's mutate seam, never raw SQL: `create_card` to `Add`, `update_card` to `Edit`, `delete_card` to `Remove`. The write is attributed to the configured `pimdir.source`; on a store not synced as that source (the placement carries no base) it SHALL fail loudly rather than stage a change no sync will carry. `update_card` ignores `--if-match`, because the engine reconciles a staged edit against the base body it recorded at sync time, which is stronger than an ETag precondition a local store cannot check.

### Requirement: pimdir derivations match the sync engine
The link id, the `v: 1` summary and the sort key a pimdir write records SHALL be derived exactly as Neverest's `text/vcard` kind derives them, and the body content hash SHALL be the shared 128-bit FNV-1a digest rendered as 32 hex chars. Both write into the same store, so a card Cardamum adds has to link, summarize, sort and deduplicate identically to the same card arriving through a sync. The derivations use a small scanner rather than a vCard parser, and the body is stored as opaque bytes, so a property the scanner does not understand cannot be lost.

### Requirement: pimdir writes auto-source
When `pimdir.source` is unset, the pimdir backend SHALL attribute its writes to the store's single synced source (via `distinct_sources`) when there is exactly one, the local-sync case, so a staged write propagates without configuration, falling back to `local` only when the store has no or several sources.

### Requirement: pimdir store path is shell-expanded
The pimdir backend SHALL expand `~` and environment variables on `pimdir.root` before opening the store and its blob reader. Opening the raw path would create an empty store at a literal `./~/…` relative to the cwd and silently return an empty addressbook list.

### Requirement: pimdir addressbook limits are explicit
`create_addressbook` SHALL declare a local `text/vcard` collection and say nothing about a remote one, since pimdir stages item mutations only. `update_addressbook` SHALL always fail, per the requirement above: io-pimdir's `rename_collection` renames the identifier rather than the display name, and the row it would touch is the sync's to write. `delete_addressbook` SHALL always fail with a message naming the alternative (delete on the server and sync, or remove the store directory), because io-pimdir exposes no collection removal and io-replica has no collection-level mutation to stage one.

### Requirement: Backend selection is local-first
The shared commands run over an `AddressbookClient` owning one `BackendClient` variant per compiled-in backend. The global `--backend` flag picks it: `auto` (the default) SHALL take the first configured-and-allowed backend in priority order, which puts the local stores (vdir, then pimdir) before the network ones so an offline store wins over a round-trip. A named value pins that backend and bails when the account has no matching config block. The protocol-specific commands build their own client and ignore `--backend` entirely.

### Requirement: CardDAV home-set resolution routes
The `[carddav]` block SHALL resolve the addressbook home set through one of three routes, in decreasing order of magic. `home` short-circuits all discovery and is used as the home set directly. `server` connects to the given context root, then walks current-user-principal (RFC 5397) and addressbook-home-set (RFC 6352). `discover` resolves a bare domain to a context root before that same walk: PACC first, then RFC 6764 (SRV record, its TXT `path`, then `.well-known`) through io-pim-discovery, with Google domains using an authenticated `.well-known` probe.

### Requirement: CardDAV probes a bare origin before walking
Because PACC and RFC 6764 can hand back a bare origin rather than the real context root (Fastmail serves contacts under `/dav/` and 404s everything else), the `server` and `discover` routes SHALL probe `.well-known/carddav` and follow its redirect before the principal walk whenever the resolved path is `/`.
