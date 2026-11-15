---
cairn: change
id: default-is-what-the-server-states
status: landed
created: 2026-11-15
---

# default is what the server states

## Why

Live testing of 2026-11-15 (reports/cardamum/jmap-fastmail.md J1, low): `addressbook list` printed `"default": false` on every JMAP, Graph and People address book (src/jmap/backend.rs, src/msgraph/backend.rs, src/gpeople/backend.rs hard-coded it), while `jmap address-book get` shows `isDefault: true` and the same Graph and Google collections read `default` on their pimdir accounts. The output type documents the key as what the server states. Task C33 of cli-alignment-2026-11-15.

## What

- JMAP: `isDefault` (RFC 9610 §2). Graph: the default Contacts folder, where a contact naming no folder goes. People: myContacts, which holds every person. CardDAV and vdir state none and stay `false`.
