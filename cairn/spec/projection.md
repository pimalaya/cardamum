---
cairn: spec
capability: projection
status: current
---

# vCard projection

CardDAV, vdir and pimdir speak vCard natively. JMAP, Microsoft Graph and Google People do not: for those three, the shared `Card.contents` is a vCard document of record that Cardamum synthesizes from the backend's own contact resource and re-projects on the way back ([jmap](../../src/jmap/project.rs), [msgraph](https://docs.rs/io-msgraph/latest/io_msgraph/v1/rest/users/contacts/vcard/) (io-msgraph's `vcard` feature), [gpeople](https://docs.rs/io-gpeople/latest/io_gpeople/v1/rest/people/vcard/) (io-gpeople's `vcard` feature), the vCard building blocks being vcard-rs API).

These modules are ported verbatim from cardamum-android, so both products treat the same provider quirks identically.

### Requirement: JMAP converts through JSContact
The JMAP projection SHALL convert a `ContactCard`'s JSContact payload (RFC 9553) through vcard-rs's RFC 9555 codec, both ways. A vCard property with no JSContact counterpart rides the standard `vCardProps` escape hatch, so the document round-trips losslessly. The non-standard `vCard` container some libraries emit SHALL NOT be produced: Fastmail rejects it.

### Requirement: Only well-slotted fields are managed
A provider field SHALL be *managed* (read into the vCard on the way out, written back on the way in) only when it has a well-defined vCard slot. A managed field is authoritative in both directions, so clearing the vCard property clears the provider field on the next update.

### Requirement: Provider-only fields are left alone
A provider field with no vCard equivalent (Graph's `fileAs`, `officeLocation`, `assistantName`, `manager`; People's `fileAses`, `memberships`, `events`, `photos`) SHALL stay out of every update mask and survive an update untouched.

### Requirement: Provider-scoped fields are minted read-only
A provider field that means nothing outside the account (Google external ids, misc keywords, locations) SHALL be *minted* as a read-only `X-GOOGLE-*` or `X-MSGRAPH-*` vendor property on read and *consumed* (dropped) on write, the server value staying authoritative. A minted property is therefore neither managed nor part of the stash remainder.

### Requirement: The remainder is stashed verbatim
Every vCard line the projection neither manages nor mints SHALL be stashed verbatim in a provider custom-data slot and spliced back on read, so a property no provider field models survives a round-trip instead of being dropped on the next write. Google People uses a `clientData` entry keyed `cardamum.vcard`; Microsoft Graph uses a `singleValueExtendedProperty` named `cardamum-vcard`. The remainder SHALL be recomputed from the incoming vCard on every write, so the stash never drifts.

### Requirement: Oversized lines stay local
A stashed line longer than `MAX_STASH_LINE` (8 KiB, in practice a base64 `PHOTO` blob) SHALL NOT be sent server-side. It stays only in the local document of record, so an oversized inline photo never risks the whole write against an undocumented provider size limit. On Google People, a create or an update leaving such a line out SHALL log a warning naming the card and each property left out (`GpeoplePersonVcard::dropped`), the receipt of the unified command staying the one every backend prints (see [backends.md](backends.md), "Unified outputs carry no backend details").

#### Scenario: A photo too long for the stash
- GIVEN a vCard whose `PHOTO` line is longer than 8 KiB
- WHEN it is created on a People address book
- THEN the request carries no `PHOTO`, and a warning reads ``card `<id>` stored without PHOTO: too long for the server``

### Requirement: A People update keeps what the vCard cannot show
A People update replaces each field of its mask whole, and a managed field holds more than its vCard shows: a further organization or occupation, a relation of another type, a custom type, a sub-field without a vCard slot, the client data of other clients. `update_card` SHALL re-attach all of it from the person it just read, in the fields of the mask only, through io-gpeople's `GpeoplePerson::merge_unprojected`, between computing the mask and sending the update, and SHALL carry no merge of its own.

#### Scenario: A second organization survives an edit of the first
- GIVEN a person holding the organizations Acme (symbol `ACM`) and Initech, and a `clientData` entry of another client
- WHEN the card is updated with `ORG` changed to Globex and a new `X-PET` line
- THEN the update sends Globex with its symbol `ACM`, then Initech, and the other client's entry beside the stash

### Requirement: Slot shape differs per provider
Microsoft Graph has *fixed* slots (a bounded set of emails, phones and IM addresses) and rejects bodies that overflow them, so the first properties win and the overflow lands in the stash remainder like any unmanaged line, surviving on the server and restoring on read. Google People fields are true lists, so every vCard property of a managed kind projects without truncation.

### Requirement: The UID rides the stash

Graph and People have no UID field, so a synthesized card SHALL take its UID from the provider stash, where every card written through Cardamum carries the one it was written with, and SHALL mint one from the provider id only for a contact the stash carries none for (created by the provider, or written by a Cardamum predating this rule). A minted UID never changes, the provider id never changing.

#### Scenario: A card created with a UID keeps it
- GIVEN a vCard with `UID:urn:uuid:4fbe8971`
- WHEN it is created on a Graph or People address book and read back
- THEN the card carries `UID:urn:uuid:4fbe8971`, not one derived from the Graph id or the People resource name
