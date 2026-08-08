---
cairn: spec
capability: addressbooks
status: current
---

# Addressbooks

What an addressbook is per backend, and why group membership is structural rather than a card property.

### Requirement: Every backend maps one container to an addressbook
An addressbook SHALL be the backend's own container: a collection directory (vdir), a `text/vcard` collection (pimdir), a CardDAV collection under the home set, a JMAP `AddressBook`, a Microsoft Graph contact folder, or a Google People contact group. Its shared `id` is that container's native identifier, and its `name` falls back to the id when the backend exposes no display name.

### Requirement: Partial-coverage metadata stays optional
`description` and `color` SHALL remain optional on the shared shape and be populated only by the backends that know them. A backend that stores neither SHALL reject a patch carrying them rather than accept and drop them.

### Requirement: Memberships are addressbook data, not card data
A contact's group memberships SHALL be surfaced structurally, as which addressbook(s) the card belongs to, never projected onto the vCard. A card can therefore appear under several addressbooks at once, matching cardamum-android's merged contact-first view assembled over per-replica storage.

### Requirement: Stale membership lines are dropped, not stashed
`X-GOOGLE-MEMBERSHIP` SHALL NOT be minted by the People projection, and SHALL stay on the *consumed* list so a line written by an earlier projection is dropped on the way back rather than stashed into the custom-data remainder (see [projection.md](projection.md)). This keeps an old document from re-injecting a stale membership line.
