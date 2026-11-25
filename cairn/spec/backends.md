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
CardDAV, JMAP, Microsoft Graph and Google People SHALL each adapt their io-* high-level client. CardDAV reuses io-webdav's addressbook and card verbs over the resolved addressbook home set. JMAP reuses io-jmap's RFC 9610 `AddressBook` and `ContactCard` methods. Microsoft Graph maps contact folders, at every depth (each folder's `childFolders` listed in turn, every page read), to addressbooks, keyed by folder id and named by display name, over io-msgraph's `contact_folders` / `contacts` surface. Google People maps contact groups to addressbooks over io-people's `contactGroups` / `gpeople` surface.

### Requirement: A shared update never creates
The unified `card update` and `addressbook update` SHALL fail when the target does not exist, rather than creating it. On a backend whose write verb is create-or-replace (CardDAV `PUT`, a filesystem write), the adapter SHALL establish that the target exists before writing: CardDAV reads the card's current version and guards the write with the ETag it returns, and vdir locates the item. An unknown id then fails on that read rather than creating anything. The wildcard `If-Match: *` SHALL NOT be used for this: RFC 9110 §13.1.1 defines it, but iCloud rejects it with `412` even for a resource that exists. A protocol-specific command SHALL keep its verb's native semantics instead: `carddav put` stays create-or-replace, gated only by the preconditions the caller passes.

### Requirement: Clearing an optional field removes it
A shared update SHALL distinguish "leave this field untouched" from "clear this field", and SHALL carry the difference all the way to the storage: on CardDAV a cleared property leaves as a `DAV:remove` instruction (RFC 4918 §9.2), never as an omitted `DAV:set`, on vdir a cleared property has its metadata file removed rather than left in place, and on JMAP a cleared description leaves as `null` in the `AddressBook/set` update. A backend that cannot express removal SHALL report that rather than accept the request and drop it. The adapter SHALL forward the diff as such, never reading the current state to merge unchanged fields by hand.

### Requirement: Unified outputs carry no backend details
The output of a unified command SHALL NOT carry a field that only one backend fills. A backend detail worth showing SHALL be logged by that backend's adapter or shown by its own namespace. The receipt key `queued` is no such field: the family's receipts carry it on every backend (pimalaya-cli output.md), `false` wherever the write reached the server.

### Requirement: An unsupported precondition is refused, not dropped
A backend with no entity-tag concept SHALL refuse `--if-match` rather than accept and ignore it: its table declares `card.update.if-match` or `card.delete.if-match` at `none`, so the command fails with `capability-missing` before connecting, and the adapter keeps its own refusal. Silently dropping a guard the caller asked for is the one outcome that reads as protection and is none. A backend whose read reports a version its write takes no guard on (the Graph `changeKey`, the hash of a JMAP ContactCard) SHALL compare it with the card it reads just before the write and declare the guard `partial`, the detail saying a write landing in between is not caught ("A delete can be gated on a version", commands).

### Requirement: Card ids are backend-native and verbatim
A card's shared `id` SHALL be the backend's own identifier, used verbatim end to end: the last path segment of the resource href, decoded as "A CardDAV id round-trips" says (CardDAV), the file stem (vdir), the `ContactCard` id (JMAP), the contact id (Graph), the person id (People), or the store-assigned public `seq` (pimdir). No adapter SHALL add or strip a file extension when addressing a resource. CardDAV names a new card `<uuid>.vcf` itself and passes the whole name through, since the server owns the resource name from then on. A CardDAV write SHALL report the id io-webdav answers, never the caller's string: `card add` and `carddav put` print the last segment of the `Location` the server names, else of the URL sent, decoded as a listing shows it.

### Requirement: A JMAP card's version is a stable digest
The JMAP backend SHALL report as a card's `etag`, in `card list` and `card read`, the BLAKE3 digest of the ContactCard's JSON in lowercase hex, its keys in serde_json's sorted order, JMAP having no per-card version. The function SHALL be a fixed one, never Rust's `DefaultHasher`, whose algorithm may change from one Rust release to the next: a version one build prints is passed back as `--if-match` to another, and compared with the digest of the card read before the write ("An update guards on the version it read" in commands).

### Requirement: A listing is whole or fails
A listing SHALL never come back shorter than the backend holds without saying so: a card it could not read SHALL be an `errors` entry ("Each backend names the cards it could not read"), and a listing that cannot be read at all SHALL fail. A CardDAV listing whose answer is not one whole `multistatus` document (an HTML page, a cut body, an empty one) SHALL fail with io-webdav's error; a CardDAV listing the server truncated (a 507 row, RFC 6352 §8.6.1) SHALL add an entry with a `null` id and no code saying the listing is incomplete. A vdir listing SHALL leave out a card removed while it lists. A Graph listing SHALL follow its next page through io-msgraph's `page_get`, which refuses a link off the Graph origin, so the token never follows a link elsewhere.

### Requirement: Each backend names the cards it could not read
A backend SHALL report a card it could not read as an `errors` entry and SHALL never drop it silently nor fail the listing for it. A card named in `errors` SHALL NOT also list among the cards ("A listing names the items it could not read"):

- every backend: a card whose body does not parse as a vCard, or is not UTF-8, SHALL be a `card-unreadable` entry naming the card, no row listed for it;
- vdir: a card file that cannot be read SHALL be a `card-unreadable` entry naming the card, its error in `sources`; a card removed while it lists SHALL be left out with no entry;
- JMAP: a card that does not project SHALL be a `card-unreadable` entry, its id when it has one;
- pimdir: a body that cannot be read SHALL be a `card-unreadable` entry, no row listed for it, though its summary reads; a body not local SHALL be no entry, the card listed with its preview, whether it was never fetched or the item names a body its blob tree no longer holds (the blob file gone, or failing its name), which is not hydrated and not an error (pimdir GUIDE §14), as Himalaya and calendula list such an item. `pimdir search` SHALL name a hit the same way, an item the store cannot read and a body that reads as no vCard included, its id the hit's;
- CardDAV: a row the server answered without the card's data (a row refused whole, a refused `address-data` property, or none sent) SHALL be an entry naming the card, coded `permission-denied` when the row or a refused property carries a 403, else `card-unreadable`, each refused property and its status in `sources`; the entries print on the first page only, as the whole address book is read before the page is cut;
- CardDAV, Graph, People: a page or an answer that fails SHALL fail the listing whole.

### Requirement: People resolves its contact group before acting
The People backend SHALL read the contact group a card command names (`contactGroups.get`, no members) before it lists, reads or writes anything through it, myContacts excepted, every account holding it: a group People does not hold SHALL fail coded `addressbook-not-found`, ``Addressbook `<id>` not found`` with People's answer in `sources`, after that one request. A listing narrows every connection to the group's members on the client, and a create lands in myContacts before its membership is added, so without it a typo lists as an empty group and a create leaves a contact behind before failing. A membership refused after the create SHALL say the card was created in Contacts and not added to the group. `addressbook update` and `addressbook delete` of a group People does not hold SHALL fail with the same code.

#### Scenario: A card added to a group Google does not hold
- GIVEN a Google account holding no contact group `nope`
- WHEN `cardamum --json card add -k nope c.vcf` runs, or `cardamum --json card list -k nope`
- THEN each exits 1 coded `addressbook-not-found` after one `GET contactGroups/nope`, and no `createContact` is sent

### Requirement: A People group delete keeps its contacts
`addressbook delete` on Google People SHALL delete the contact group alone (`contactGroups.delete` with `deleteContacts=false`), its contacts staying in myContacts: a contact may sit in other groups, and every contact sits in myContacts. The help of `addressbook delete` and the `addressbook.delete` detail of People (`partial`) SHALL say the contacts stay. `gpeople contact-groups delete --delete-contacts` deletes them on request.

#### Scenario: A group deleted on People
- GIVEN a Google account holding the contact group `g1` with the contact `c1`
- WHEN `cardamum --json addressbook delete g1` runs
- THEN it prints `{"id":"g1","queued":false}`, `g1` is gone and `c1` stays in myContacts

### Requirement: An other contact copy sends the copy mask
`gpeople other-contacts copy` SHALL send People the copy mask `names,emailAddresses,phoneNumbers`, the three fields `otherContacts.copyOtherContactToMyContactsGroup` copies, never the wider read mask the other-contacts listings send: People refuses a copy mask naming `metadata` with `INVALID_ARGUMENT`. The person it prints is read with the full person mask.

#### Scenario: An other contact copied on People
- GIVEN a Google account holding the other contact `otherContacts/o1`
- WHEN `cardamum gpeople other-contacts copy otherContacts/o1` runs
- THEN the request body names `copyMask: names,emailAddresses,phoneNumbers` and the copied person is printed

### Requirement: A network backend names an unknown address book
A unified command naming an address book a CardDAV server, JMAP or Microsoft Graph does not hold SHALL fail coded `addressbook-not-found`, ``Addressbook `<id>` not found``, the server's answer in `sources` ("A command resolves its addressbook before acting" in commands), never with the server's bare refusal: on CardDAV a 404 or a 410 of the collection, on `card list`, `addressbook update` and `addressbook delete`; on Graph a 404 or a 400 `ErrorInvalidIdMalformed` of the folder, on `card list`, `card add`, `addressbook update` and `addressbook delete`; on JMAP a `card list` whose `ContactCard/query` fails or finds nothing SHALL ask `AddressBook/get` for the id, one request on that path alone, an id the account does not hold failing so with the query's refusal in `sources`, an empty listing of one it holds staying empty and a refusal over one it holds staying the refusal. A server answering an unknown collection otherwise (iCloud's 400 `invalid collection ID URI`) keeps its refusal.

#### Scenario: An unknown address book on JMAP
- GIVEN a JMAP account answering a `ContactCard/query` filtered on `nope` with `invalidArguments` and holding no such address book
- WHEN `cardamum --json card list -k nope` runs
- THEN it exits 1 coded `addressbook-not-found`, ``Addressbook `nope` not found``, the `invalidArguments` refusal in `sources`, after the query and one `AddressBook/get`

### Requirement: A network listing reads the page it prints
The unified `card list` on JMAP SHALL ask the server the window it prints (`position`, `limit` the page size plus one), in chunks no larger than the session's `maxObjectsInGet`, reading `total` to tell a clamped answer from the end; on Microsoft Graph it SHALL send `$skip` and `$top` (at most 100) and follow `@odata.nextLink` only until it holds the window; on Google People, which has no offset, it SHALL read the connections in pages of 1,000, the API's largest, and stop once it holds the cards before and in the window. Page 1 of a large address book SHALL cost one request on each.

### Requirement: A refused add names a missing address book
`card add -k <id>` into an address book the server does not hold SHALL fail coded `addressbook-not-found`, the server's refusal in `sources`: on CardDAV a PUT answered 404 or 410, or 409 naming no precondition (RFC 4918 §9.7.1); on JMAP a refused create (no code, or `notFound`) after one `AddressBook/get` shows the book missing. A stale People `etag` (400 `FAILED_PRECONDITION`, or 412) SHALL be `precondition-failed`, as on the four other network backends, an expired sync token excepted. A JMAP method error inside a typed call (`forbidden` on `card list`) SHALL carry its code as an HTTP refusal does.

### Requirement: A card verb acts only in the address book it names
`card read`, `card update` and `card delete` on JMAP, Microsoft Graph and Google People SHALL act only on a card the address book `-k` names holds, a card held elsewhere failing coded `card-not-found` with nothing written or deleted, though each server reads a card by id whatever its address book. JMAP SHALL check the card's `addressBookIds`, Graph its `parentFolderId` (the default Contacts folder resolved as the parent of the folders `GET /me/contactFolders` lists, every contact being in it when it lists none), People its contact group memberships (myContacts holding every contact). A delete SHALL read the card first, at one request more where no `--if-match` asked for that read.

#### Scenario: A card of another address book
- GIVEN a Graph account whose contact `AAMk` is filed in the folder `B`
- WHEN `cardamum --json card delete -k A AAMk` runs
- THEN it exits 1 coded `card-not-found`, one `GET` sent and no `DELETE`

### Requirement: Local storage backends
vdir and pimdir SHALL adapt io-vdir and io-pimdir. vdir stores each addressbook as an immediate subdirectory of `vdir.root` and each card as a `.vcf` file inside it, byte-faithfully. The unified card verbs (`card read`, `update`, `delete`) SHALL act on a `.vcf` item alone: an id naming an `.ics` item the collection also holds is `card-not-found`, as is an id no item can have (`a/b`), never read as a card, rewritten beside a `.vcf` or removed. A listed file opening no `BEGIN:VCARD` SHALL be a `card-unreadable` entry, not a row of nulls. pimdir is an offline cache a sync engine populates through io-pimdir, which holds both the store and the engine, and follows the cache requirements below.

### Requirement: vdir refuses an unknown addressbook
A vdir addressbook id SHALL name an addressbook only when it is one directory name, one path segment that is neither `.` nor `..` and holds no separator (a trailing one included), naming a directory under the home. Every vdir command naming another SHALL fail before anything is read or written, coded `addressbook-not-found`: `addressbook update`, `addressbook delete` and every `card` read and write with ``Addressbook `<id>` not found``, `vdir delete`, `vdir rename` and every `vdir item` command with ``Collection `<id>` not found``. A new name that is not one directory name SHALL be refused before anything is written: `addressbook create` and `vdir create` uncoded, the new name of `vdir rename` by io-vdir.

A write creates whatever directory it is handed, a listing of a missing one answers nothing and a delete removes whatever the path reaches, so without it a typo in `-k` would read as an empty addressbook or invent one, and an id reaching out of the home (`..`, `../keep`, `friends/..`) would remove a directory there.

#### Scenario: An id reaching out of the home
- GIVEN the account `perso` whose vdir home `outer/home` holds the addressbook `friends` alone, beside the directory `outer/keep`
- WHEN `cardamum --json addressbook delete ..`, `addressbook delete ../keep` or `card list -k nope` runs
- THEN each exits 1 with the failure object coded `addressbook-not-found`, and the home, `outer/keep` and the directory holding them are left as they were

### Requirement: A collection created twice is refused
`addressbook create` naming an address book the backend holds already SHALL fail coded `addressbook-duplicate`, ``Cannot create address book `<id>`: it exists already``, `<id>` the name or the id `--id` gave, as Himalaya words a mailbox created twice, exit 1, creating nothing and leaving the existing one as it was, on every backend that can tell, as pimalaya-cli's error.md asks of every collection noun. vdir: a directory of that name under the home, checked before anything is written, `vdir create` refused the same way, where io-vdir would create over it and rewrite its metadata. CardDAV: a `MKCOL` refused 405, the status RFC 4918 §9.3.1 gives a URL already mapped and RFC 5689 §3.2 keeps for the extended `MKCOL`, or 403, which Fastmail answers (testing/carddav-specific.md), on an id the home set then lists, one `PROPFIND` of the home set on that path alone; any other refusal, or a listing that fails or does not hold the id (iCloud refuses every `MKCOL`), keeps the server's. Microsoft Graph: a folder refused `ErrorFolderExists`. Google People: a contact group refused 409, which contactGroups.create answers a name the user holds. The server's answer SHALL be in `sources`. JMAP lets two AddressBooks share a name (RFC 9610 §2 asks no uniqueness), so it holds no duplicate to refuse, and pimdir creates no address book (`capability-missing`, "A cache renames nothing it cannot push"). The protocol commands creating a collection on a server (`carddav mkcol`, `msgraph contact-folders create`, `gpeople contact-groups create`, `jmap address-book create`) keep the server's refusal as it is.

#### Scenario: An address book created twice
- GIVEN a vdir home holding the address book `work`, named `Work` and described
- WHEN `cardamum --json addressbook create work` or `cardamum --json vdir create work -n Other` runs
- THEN each exits 1 coded `addressbook-duplicate`, ``Cannot create address book `work`: it exists already``, and `work` keeps its name and description

`vdir rename <OLD-ID> <NEW-ID>` SHALL be refused the same way when NEW-ID is one directory name the home holds already, a directory (full or empty) or any other entry, coded `addressbook-duplicate`, ``Cannot rename collection `<old-id>` to `<new-id>`: it exists already``, exit 1, before anything moves: the rename would otherwise fail on a full directory with the raw OS error and replace an empty one. A NEW-ID that is not one directory name stays io-vdir's refusal.

#### Scenario: A collection renamed onto another
- GIVEN a vdir home holding the address books `work` and `perso`, and an empty directory `empty`
- WHEN `cardamum --json vdir rename work perso` or `cardamum --json vdir rename work empty` runs
- THEN each exits 1 coded `addressbook-duplicate`, ``Cannot rename collection `work` to `perso`: it exists already`` (or `empty`), and the three are left as they were

### Requirement: A JMAP account may forbid address book writes
JMAP SHALL declare `addressbook.create` and `addressbook.update` `server`, a JMAP account being allowed to create no AddressBook (RFC 9610 §1.4.1 `mayCreateAddressBook`) and a server refusing the rename too, as Fastmail does. JMAP SHALL declare `addressbook.delete` `server`, an AddressBook stating whether it may be destroyed (RFC 9610 §2 `myRights.mayDelete`), as Fastmail's Personal states `false`; the server's refusal is kept, no `AddressBook/get` read first. `addressbook create` SHALL be refused coded `permission-denied`, ``The JMAP account may not create address books (`mayCreateAddressBook`)``, before any `AddressBook/set`, when the session's contacts capability of the contacts account states `mayCreateAddressBook: false`; an account stating nothing is left to the server. `jmap address-book create` keeps the server's refusal as it is.

#### Scenario: An account that may not create
- GIVEN a JMAP account whose contacts capability states `mayCreateAddressBook: false`
- WHEN `cardamum --json addressbook create Work` runs
- THEN it exits 1 coded `permission-denied`, and no `AddressBook/set` is sent

### Requirement: A cache renames nothing it cannot push
The pimdir backend SHALL refuse every `addressbook update` field. Its collection row (id, display name, description, colour) is written by the sync from the server, and the backend stages item actions only, so any local edit of it is a change no sync would carry. A body not fetched yet SHALL fall back to the summary preview in a listing, rather than render as a blank row; a body the item names but the store no longer holds is named in `errors` ("Each backend names the cards it could not read").

### Requirement: A pimdir addressbook is its collection id
The pimdir backend SHALL show and accept an address book as the store's collection id, verbatim: the collection `carddav/default` is the address book `carddav/default`. It SHALL NOT derive, strip or accept a shortened spelling, and no configuration SHALL offer one.

A sync engine binds a source's collections under a namespace, so an id carries one; the store is opaque to it, neither parsing nor validating an id (pimdir STORAGE §9.2) and modelling hierarchy through `parent` rather than through a separator. Shortening is therefore a guess at the producer's convention, and one that makes a single address book answer to two spellings.

An id SHALL name an address book or name nothing: one store holds the collections of every kind a sync caches, so the kind SHALL narrow them at the one seam both the listing and the id check read, never at the listing alone. The kind SHALL be compared on its media type, read by io-pimdir's `collection::media_type` (the part before any `;` parameter, trimmed), the one reading io-pimdir applies when it picks a collection's summaries. An undeclared collection, of the empty kind, SHALL belong to no application's kind: it is no address book.

An id naming no address book of the account SHALL be refused naming the ones it holds. Ids carry the sync engine's namespace and are not guessable, so an error asking for one that shows none leaves the user nothing to act on.

### Requirement: pimdir is a cache, not a server
The pimdir backend SHALL treat the store as a possibly-partial cache. `get_card` on a card whose body is not local SHALL fail coded `body-pending`, ``Card `<id>` in `<addressbook>` is not downloaded yet (body not fetched); run a sync to hydrate it``, the cue to sync, rather than a data-loss error, exit 1, as Himalaya reads such a message: a card never fetched (`level < Full`, no stored object) and one whose blob file is gone or fails its name alike (pimdir GUIDE §14). The card still lists: `list_cards` SHALL project the stored contact summary (pimdir STORAGE Annex A) into a minimal preview vCard (`UID`, `FN`, `EMAIL`), the name escaped back into a content line, so a contact list reads correctly before a full sync, while `get_card` refuses outright so a preview can never be mistaken for the document of record. A body the store holds but cannot read (an I/O error) SHALL fail `get_card`, and an update or a delete given `--if-match`, coded `card-unreadable`, ``Cannot read card `<id>` in address book `<addressbook>` ``, the store's error in `sources`, as `card list` codes its entry; an update or a delete given no version SHALL read no body, so such a card can still be replaced or removed.

#### Scenario: A body the store cannot read
- GIVEN a pimdir address book holding a synced card whose blob file has no read permission
- WHEN `cardamum --json card read <id>` runs
- THEN it fails coded `card-unreadable`, its error naming the card and the address book, the permission error in `sources`, exit 1, as `card list` names it in `errors`
- AND `card delete <id>` with no `--if-match` stages its removal

#### Scenario: A body gone from the blob tree
- GIVEN a pimdir address book holding a synced card whose blob file was removed
- WHEN `cardamum --json card list` runs
- THEN it lists the card with its preview and `etag` `null`, `errors` empty, exit 0
- AND `cardamum --json card read <id>` fails coded `body-pending`, exit 1

### Requirement: A pimdir card listing streams the store
`list_cards` on pimdir SHALL walk the address book in keyset batches of the store's contacts order and hold one batch of rows and the page it returns, never the address book. The cards before the page SHALL be counted off their rows without reading a body, a body SHALL be read only for a card the page returns, and the walk SHALL stop once the page is full.

### Requirement: pimdir takes the reader and producer roles, the owner's for one cancellation
The pimdir backend SHALL read through a `PimdirReader` and write through a `PimdirProducer`, and SHALL open a `PimdirStore` for `pimdir queue cancel` alone, through `PimdirStore::open_current`, which creates and migrates nothing, dropping the one row and releasing the handle inside the call, as Himalaya cancels a queued action. The owner handle drains the queue, sweeps the objects and purges the trash, and holds an exclusive lock on the store for its lifetime, so holding it any longer would both lock a sync out for the length of a command and put every destructive verb behind a frontend that never calls them; a sync holding it refuses the cancellation at once, coded `store-owned`. The reader SHALL be built with the pending overlay, so an update or removal this process staged reads back before the store's owner applies it.

### Requirement: A pimdir body is named by the store's own hash
The pimdir backend SHALL leave a body's name to the producer staging it (`enqueue_with_body`), which hashes it with BLAKE3, the one algorithm a store records in `store_meta.hash_algo` (pimdir STORAGE §5; a store recording another is refused as `store-hash-algo`), and SHALL NOT hash or write a body itself. A body named under another algorithm is a body no read ever finds, and one written outside the producer's staging lock is one a collector may remove before its row pins it.

### Requirement: pimdir writes are staged queue actions
A pimdir write SHALL append one action to the store's queue (pimdir STORAGE §15.1) through a producer opened for that write and dropped after it: `create_card` to `add`, `update_card` to `update`, `delete_card` to `remove`. A write carrying a body goes through the producer's own `enqueue_with_body`, which checks the action, removes every `X-PIMDIR-` property from the card (pimdir STORAGE §18: no instruction to a performer enters the queue unasked, and no card instruction is the user's to ask), then writes the body under its staging lock and appends the row pinning it in one transaction, so a refused write leaves no body behind; a `remove` goes through `enqueue`. Each runs the producer's gate once (pimdir STORAGE §15.6), cardamum running none of its own before it, and answers the partial supports the gate passed the action with, each logged as a warning. The action addresses the item by the public `seq` that is already the card's shared id. `update_card` and `delete_card` SHALL honour `--if-match` against the card's version; the engine still reconciles an applied edit against the base body it recorded at sync time: the precondition is the caller's, the merge the store's. Because a queued create carries no public id until the owner applies it, `create_card` SHALL report the card's link id instead.

A staged update or removal SHALL read back at once, the reader folding the pending queue over the stored rows. A staged create SHALL NOT be listed until the owner applies it: it has no public id to list under, and no placeholder SHALL stand in for one.

### Requirement: A pimdir write reaches a synced address book or is refused
The pimdir backend SHALL refuse `create_card`, `update_card` and `delete_card`, before anything reaches the blob tree or the queue, on an address book no source syncs: one for which the store records neither a source checkpoint nor a binding (`collection_sources`), with the code `no-performer` (himalaya's and calendula's code for the same case) and a message naming the address book and saying to run a sync first. The owner applies a queued `add` there as whichever source drains first, binding it to a collection that source never lists, so the card would never be pushed; an `update` of a card no source binds applies to the store alone and a `remove` retains the card, neither reaching a server. A collection the device keeps for itself is the owner's to write (pimdir STORAGE §14.3), never a producer's.

#### Scenario: A write into an address book no source syncs
- GIVEN a pimdir store holding the address book `local`, which no source syncs
- WHEN `cardamum --json card add -k local card.vcf` runs
- THEN it exits 1 with the code `no-performer`, the queue and the blob tree unchanged

### Requirement: A duplicate pimdir create is refused
`create_card` on pimdir SHALL refuse a card whose link id the address book already holds, as a live item or a pending create, appending nothing to the queue and writing nothing to the blob tree, with the code `item-duplicate`. The refusal SHALL be io-pimdir's (`PimdirProducer::enqueue_with_body`, checked in the transaction that would append the row, before the body is written), never a lookup of cardamum's own. The owner would park such an `add` at drain (pimdir STORAGE §15.3), so accepting it reports a success no sync carries out. With `-i`, the composer file is kept and named, as on any failed write.

### Requirement: A pimdir view is one account
The pimdir backend SHALL read one account of the store. With `pimdir.account` set it reads that one, a name the store groups no collection under being refused coded `store-account-unknown` when the store groups any, ``The pimdir store holds no account `<name>` (<accounts>); fix `pimdir.account` ``, rather than read as an empty account, and accepted on a store holding no collection yet, a first sync not having run; unset, it SHALL derive the account from the store when its address books (collections of kind `text/vcard`, an undeclared collection of the empty kind not one) are grouped under one account or none, an account holding no address book not counted, as Himalaya counts the accounts holding mail, and SHALL refuse a store whose address books several accounts group, coded `store-account-ambiguous`, naming them, rather than show their address books as one. The two codes are Himalaya's. A view mixing accounts is the Android and Linux apps' to offer, not a CLI's.

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
The link id, summary and sort key of a card a pimdir write stages SHALL be io-pimdir's own derivations (`io_pimdir::summary`, pimdir STORAGE Annex A), and the backend SHALL carry none of its own. A queued `add` or `update` names the body alone: the store's owner derives the summary and the sort key from it when it applies the action, so a card Cardamum stages summarizes exactly as the same card arriving through a sync. The link id `create_card` reports is the same derivation run on the bytes the store holds, every `X-PIMDIR-` property taken out, and is the key io-pimdir reports for the queued `add` (`PimdirEnqueued::link_id`); the action states a `UID`-derived key, checked against the cards held or queued, and never a `hash:` one, which io-pimdir derives itself: the bare `UID`, or `esc:` and the `UID` when it begins with a prefix the store reserves for the keys it derives (`alt:`, `hash:`, `dup:`, `part:`, `file:`, `esc:`, pimdir STORAGE §9), or `hash:` over the body when the card states none, or one that is empty or of spaces and tabs alone.

A derivation is what a write carries, not a lookup: the store keys an item it received from a source by the derived id only when that id is free in the collection, and mints a distinct one otherwise (pimdir STORAGE §9). Reading a card by re-deriving its `UID` is therefore not addressing, and the public `seq` is.

### Requirement: A store refusal carries its code
The pimdir backend SHALL open a store as the io-pimdir it builds against defines it and SHALL NOT migrate one, migrating being the owner's. A failure to open, read the queue of, or stage into the pimdir store SHALL carry, under `--json`, the code io-pimdir's `PimdirError::code()` names for it, spelled as io-pimdir spells it, for the failures a caller acts on: `store-uncreated`, `store-version`, `store-foreign`, `store-corrupt`, `store-hash-algo`, `store-exposed`, `store-busy`, `lock-replaced`, `unsupported`, `item-duplicate` and `action-malformed`, and every other code Himalaya relays, so one store failure reads the same code in every program of the family: `store-not-current`, `sync-version`, `store-owned`, `store-staging`, `store-in-flight`, `store-reset`, `flag-invalid`, `conflict-unread`, `no-performer`, `ambiguous-performer`, `index-zone-unknown` and `tag-refused`, though no cardamum path raises some of them today (cardamum stages no flag, no intent and no tag). An SQL, I/O or JSON failure carries none, and so does a code no program of the family relays. `action-malformed` reaches cardamum as an item error of `pimdir queue list`, one per pending row another program queued with a payload its schema refuses, the other rows listed: the owner's drain parks it. `lock-replaced` reaches cardamum on a write, the producer checking before its enqueue that the staging lock file it holds is still the one at its path (pimdir STORAGE §8): nothing is queued. The message is io-pimdir's, `store-version` naming cardamum (see "A pimdir store of another version is refused in io-pimdir's words"), and SHALL NOT advise deleting the store: a resync restores neither the queue nor what the store keeps alone.

### Requirement: A server's refusal carries its code
A failure a server refused SHALL carry a code of the vocabulary Himalaya shares, read from the typed error of the protocol that answered it, in the failure object, an `errors` entry and an `account check` row alike. One reader per backend codes its typed errors, walking the error chain after any coded error, so the typed errors stay in the chain the backends return; the failure object is coded once the command failed, its messages and causes printing as before. A protocol command (`carddav`, `jmap`, `msgraph`, `gpeople`) the server refused prints the refusal as its result rather than the failure object, its `refusal.code` that same code (commands, "A protocol refusal is a result"); a code a protocol command puts on a typed refusal (`card-not-found`, `addressbook-not-found`) keeps that typed error in the chain, so the refusal still reads it.

| Code | When |
|---|---|
| `permission-denied` | an HTTP 403 that is no rate limit (Graph, People); a CardDAV 403 whose body names no precondition nor the vCard, or `need-privileges` (RFC 3744 §7.1.1); a CardDAV listing, `report query` or `report multiget` row, or its refused `address-data`, answered 403; a JMAP `forbidden`, a SetError or a method error, and a JMAP HTTP 403, whichever request it answered; a JMAP `addressbook create` on an account stating `mayCreateAddressBook: false`, refused before any request |
| `quota-exceeded` | an HTTP 507; a CardDAV body naming `quota-not-exceeded` (RFC 4331 §6); Graph's `ErrorQuotaExceeded`; a JMAP `overQuota` |
| `rate-limited` | an HTTP 429, or a People 403 naming a rate limit or a quota, or a JMAP `rateLimit`, a JMAP HTTP 429 included; cardamum retries nothing, so the first one met |
| `card-not-found` | a card request answered 404 or 410 (CardDAV, People), a `carddav report multiget` row included, 404 or 400 `ErrorInvalidIdMalformed` (Graph), `notFound` (JMAP ContactCard); a vdir card file or a pimdir card that does not exist, or a pimdir card id that is no number; on the protocol commands naming one card as on the unified ones, the server's message kept there |
| `addressbook-not-found` | an address book named that does not exist (vdir, pimdir); a JMAP AddressBook `notFound`; a CardDAV collection answered 404 or 410, a Graph folder 404 or a malformed folder id, a People group 404, an id JMAP's `AddressBook/get` does not hold ("A network backend names an unknown address book"); on the protocol commands naming one Graph folder as on the unified ones, Graph's message kept there |
| `addressbook-duplicate` | `addressbook create` refused on an address book the backend holds ("A collection created twice is refused"): a CardDAV `MKCOL` answered 403 or 405 on an id the home set lists, Graph's `ErrorFolderExists`, a People contactGroups.create answered 409; a `vdir rename` onto an entry the home holds |
| `card-invalid` | a CardDAV write answered 400 or 403 whose body names the `valid-address-data` or `supported-address-data` precondition (RFC 6352 §6.3.2.1) or the vCard itself (`VCARD parse error`), the predicate cardamum's hint reads, so the two never disagree |
| `precondition-failed` | a CardDAV write answered 412, the `If-Match` it sent naming a version the card no longer has |
| `item-duplicate` | a CardDAV write refused for the address book's `no-uid-conflict` precondition (RFC 6352 §6.3.2.1), as calendula codes the CalDAV one |

A refusal none of these name SHALL carry no code, its reason kept in `sources`. A CardDAV 403 naming another precondition is a card the server would not take, no matter of rights, and SHALL NOT read `permission-denied`: `no-uid-conflict` is `item-duplicate`, `valid-address-data` `card-invalid`, keeping the server's answer as the cause of cardamum's hint, and any other carries none.

#### Scenario: A vCard the server refuses
- GIVEN a CardDAV server answering a `PUT` 403 with the plain body `VCARD parse error`
- WHEN `cardamum --json card add -k card c.vcf` runs
- THEN it exits 1 coded `card-invalid`, its error the hint naming `UID` and `N`, the server's answer in `sources`

#### Scenario: A read-only address book
- GIVEN a Graph contact folder the account may read but not write
- WHEN `cardamum --json card delete <id>` runs
- THEN it exits 1 printing a failure object coded `permission-denied`

### Requirement: A pimdir store of another version is refused in io-pimdir's words
The pimdir backend SHALL open a store of the format version the io-pimdir it builds against reads (`io_pimdir::sql::FORMAT_VERSION`, `storage-draft-17` in io-pimdir 0.20.0) and no other, earlier or later, refusing it as `store-version` and reading nothing in it (pimdir STORAGE §4.2, §6). Nothing migrates a draft store. The refusal's message SHALL be io-pimdir's one wording, `version::PimdirFormatRefusal` naming `cardamum` as the program, so every program of the family says it alike: it names the version the store records, or that it records none, and the one cardamum reads, and orders them by STORAGE §4.2. A store an earlier sync engine wrote is synced with that engine until nothing is queued, then its directory moved aside and a new store synced; a store a later engine wrote asks for a later cardamum. It SHALL NOT advise deleting the store: the queued writes live in that store alone.

#### Scenario: A store an earlier sync engine wrote
- GIVEN a pimdir store whose `store_meta.format_version` is `storage-draft-15`, as io-pimdir 0.19 wrote it
- WHEN any cardamum command opens it
- THEN it fails with the code `store-version` and a message naming `storage-draft-15`, `storage-draft-17`, an earlier sync engine, the queue synced until nothing is queued and the directory moved aside

#### Scenario: A store a later sync engine wrote
- GIVEN a pimdir store of a later draft than cardamum reads
- WHEN any cardamum command opens it
- THEN it fails with the code `store-version` and a message asking to upgrade cardamum

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
The unified commands run over an `AddressbookClient` owning one `BackendClient` variant per compiled-in backend. The global `--backend` flag picks it: `auto` (the default) SHALL take the first configured-and-allowed backend in priority order, which puts the local stores (vdir, then pimdir) before the network ones so an offline store wins over a round-trip. A named value pins that backend and bails when the account has no matching config block. The order SHALL walk every block the account declares, the backends the build leaves out included, so the backend `auto` resolves to is the same in every build. The protocol-specific commands build their own client and ignore `--backend` entirely. The flag's help SHALL read as Himalaya's and list its possible values, `auto` then every backend an account may name in that order, built or not, each with Himalaya's line (``Pin the command to the account's <protocol> backend``); a value outside them SHALL be refused as a command line (`usage-invalid`, exit 2), the possible values listed.

#### Scenario: An unknown backend
- GIVEN any build
- WHEN `cardamum -b nope card list` runs
- THEN it exits 2 and the refusal lists `[possible values: auto, vdir, pimdir, carddav, jmap, msgraph, gpeople]`

### Requirement: A backend left out of the build is named
When the backend `auto` resolves to is one the build leaves out, the unified commands and `account capability` SHALL fail coded `backend-not-built`, the message naming the cargo feature ("This build leaves out the `gpeople` backend, rebuild with `--features gpeople`"), rather than take the next declared backend or report none configured. `--backend` naming a backend the build leaves out SHALL be refused as a command line (`usage-invalid`, exit 2), with the same message. Under `auto`, `account check` SHALL report each declared backend the build leaves out as a failed row coded `backend-not-built`, at its place in the resolution order. `account list` SHALL name under `--json` the declared backends the build carries in `backends`, each a value `--backend` takes, and those it leaves out in `notBuilt`, both in the resolution order; the text SHALL print both in one cell, the built ones then those left out, each left-out one marked `(not built)` ("The account outputs are the family's", commands).

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
Because a bare origin is not necessarily the context root (Fastmail serves contacts under `/dav/` and 404s everything else), the `server` route SHALL probe `.well-known/carddav` and follow its redirect before the principal walk whenever the server names no path and its scheme is `https` (the probe opens TLS, which a plain `http` port never answers), and SHALL follow it only when its target is `https`, ignoring any other with a warning: whoever serves the domain's web root would otherwise choose where the credential goes, `http` included. A `server` naming no scheme (no `://`) SHALL read as `https://` followed by it; one naming a scheme SHALL read as written, refused when it does not parse or names another scheme than `http` or `https`, never prefixed, so `https://host:99999` is refused rather than read as the host `https`.

### Requirement: A CardDAV id addresses its own resource
An address book or card id a CardDAV command names SHALL be refused, before the account's secret is resolved or anything connects, when io-webdav refuses it (`rfc4918::check_collection_id`, `check_resource_id`: empty, or holding an empty, `.` or `..` segment once split on `/` and `\` and percent-decoded, an address book id's trailing slashes and an absolute id's leading one aside), such an id addressing the home set, its parent or the server root: an address book id coded `addressbook-not-found`, ``Addressbook `<id>` not found``, a card id coded `card-not-found`, ``Card `<id>` not found in address book `<book>` ``, io-webdav's reason in `sources`, as vdir refuses an id that is not one directory name. Every unified command naming one SHALL refuse it so (`addressbook update` and `delete`, `card list`, `read`, `add`, `update` and `delete`), and every `carddav` command (`propfind`, `proppatch`, `get`, `put`, `delete`, `report query`, `multiget`, `sync` and `raw`). The name of an address book to create, its id on CardDAV, SHALL be refused the same way, uncoded as vdir refuses a new name: `addressbook create` and `carddav mkcol` with ``Addressbook name `<name>` names no CardDAV collection``, or ``Addressbook name cannot be empty``.

io-webdav refuses such an id itself before sending anything, but as an I/O error no code reads, and only once the account is open.

#### Scenario: An id climbing out of the home set
- GIVEN a CardDAV account whose password command would leave a file behind
- WHEN `cardamum --json addressbook delete ..` or `cardamum --json card read -k book ..` runs
- THEN it exits 1 coded `addressbook-not-found` (``Addressbook `..` not found``) or `card-not-found` (``Card `..` not found in address book `book` ``), the password command never ran and nothing connected

### Requirement: A CardDAV id round-trips
An address book or card id a CardDAV command prints SHALL be the one io-webdav reads off the server's href or the URL a write sent: its last segment, percent-decoded when the decoding encodes back to that segment, else as served (an encoded `/`, an escape the decoding leaves, `%40`, lowercase hex). It holds in every listing (`addressbook list`, `card list`, `carddav propfind`, `report query` and `report multiget`, their `errors` entries included) and in the id a write answers (`card add`, `carddav put`); `report sync` and `report raw` print the server's hrefs as served. An address book created by name SHALL print that name as its id, or the id `--id` gave, the one its listing shows, and every id printed SHALL address its resource when given back, io-webdav encoding it as it sends it.

#### Scenario: An address book named with a space
- GIVEN a CardDAV home set `/dav/`
- WHEN `cardamum --json addressbook create "My Book"` runs, then `cardamum --json addressbook list`
- THEN the create sends `MKCOL /dav/My%20Book/` and prints `"id":"My Book"`, and the listing, whose href is `/dav/My%20Book/`, prints `"id":"My Book"` too

### Requirement: A CardDAV card reads whole
A card a CardDAV listing (`card list`, `carddav report query`) or `carddav report multiget` reads SHALL carry the whole `address-data` text, the final CRLF a server escapes as `&#13;` included, byte-identical to the body `card read` and `carddav get` return.

### Requirement: The connection is opened by the call that needs it
`AddressbookClient` SHALL select its backend from the configuration without connecting, and open the connection on the first call that needs one. A command that never reaches the network therefore never opens a socket, and a command that runs a composer holds none open while the editor is up.

That last case is why the rule exists rather than being an optimization. A server closes an idle connection, and an editor session lasts minutes: a write landing after one reads the end of a socket whose far side is gone, and reports `unexpected end of file` for a card that is perfectly good. `card add -i` connects for the first time when it creates, after the editor has exited.

A command that must read before the editor SHALL drop that connection before spawning the composer, so the write after it opens a fresh one. `card update -i` is the only such command: it reads the card to seed the editor with it.

### Requirement: A closed CardDAV connection is reopened
Every CardDAV request SHALL run through io-webdav's client: one of its methods, or `WebdavClientStd::run` for a request it has no method for (the `carddav report raw` REPORT, the `PROPFIND` `account check` sends a `carddav.home`). A request meeting a connection the server closed before answering it (a keep-alive timed out, `Connection: close`) is then sent again once on a fresh connection, as io-webdav sends its own. No command SHALL write to or read from the client's stream by hand.

#### Scenario: A raw report after the server closed the connection
- GIVEN a CardDAV home set `/dav/` whose server closed the connection the client holds
- WHEN `cardamum --json carddav report raw "My Book" '<x/>'` runs
- THEN the REPORT is sent on that connection, then the same request on a fresh one, whose multistatus prints, where the command failed with `reached unexpected EOF`
