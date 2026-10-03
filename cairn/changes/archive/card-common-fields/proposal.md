---
cairn: change
id: card-common-fields
status: landed
created: 2026-10-03
---

# A card lists and reads with its common fields

## Why

A client driving cardamum as its contacts layer (MOA, over a pimdir store neverest syncs) shows a contact list: names, every address and number, organization, title and note. `card list` gave the first `FN`, `EMAIL` and `TEL` alone, scanned off raw lines, so a folded line, an escaped comma or a second address were lost, and the rest meant a `card read` per card plus a vCard parser of the client's own.

## What changes

- `card list` and `card read` add the common fields to their JSON, read through vcard-rs's decoded model: `uid`, `fullName`, `givenName`, `familyName`, `emails`, `phones` (a `tel:` URI as its number), `organization`, `organizationUnits`, `title`, `note`.
- The existing `fnValue`, `email` and `tel` stay, as before.
- A pimdir round trip (create, sync, list, update, read) is tested against a real store.

## What does not change

The vCard is the record and is never rewritten by a projection. The table, the field flags and every write are as they were. On pimdir, a card not downloaded yet projects what its summary knows.
