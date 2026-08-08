---
cairn: spec
capability: projection
status: current
---

# vCard projection

CardDAV, vdir and pimdir speak vCard natively. JMAP, Microsoft Graph and Google People do not: for those three, the shared `Card.contents` is a vCard document of record that Cardamum synthesizes from the backend's own contact resource and re-projects on the way back ([jmap](../../src/jmap/project.rs), [msgraph](../../src/msgraph/project.rs), [people](../../src/people/project.rs), with the shared helpers in [project.rs](../../src/project.rs)).

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
A stashed line longer than `MAX_STASH_LINE` (8 KiB, in practice a base64 `PHOTO` blob) SHALL NOT be sent server-side. It stays only in the local document of record, so an oversized inline photo never risks the whole write against an undocumented provider size limit.

### Requirement: Slot shape differs per provider
Microsoft Graph has *fixed* slots (a bounded set of emails, phones and IM addresses) and rejects bodies that overflow them, so the first properties win and the overflow lands in the stash remainder like any unmanaged line, surviving on the server and restoring on read. Google People fields are true lists, so every vCard property of a managed kind projects without truncation.
