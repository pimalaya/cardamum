---
cairn: change
id: an-unknown-addressbook-is-named
status: landed
created: 2026-11-15
---

# An unknown addressbook is named

## Why

Live testing of 2026-11-15 (carddav-fastmail.md D1, jmap-fastmail.md J2, msgraph-microsoft.md, low): `card list -k nope` failed with the server's bare refusal and `code: null`, the book not named: Fastmail's CardDAV 404, Fastmail's JMAP `invalidArguments [filter/inAddressBook]` (the query ran before the book was resolved), Graph's 400 `ErrorInvalidIdMalformed`. vdir, pimdir and People already fail `addressbook-not-found`, commands.md "A command resolves its addressbook before acting". Task C32 of cli-alignment-2026-11-15.

## What

- `error::addressbook_not_found`, cardamum's sentence over the server's answer (People's `group_error` now uses it).
- CardDAV: a 404 or 410 of the collection on `card list`, `addressbook update` and `addressbook delete`.
- Graph: a 404 or a malformed folder id on `card list`, `card add`, `addressbook update` and `addressbook delete`.
- JMAP: a `card list` whose query fails or finds nothing asks `AddressBook/get` for the id, one request on that path alone; an id the account does not hold fails so, an empty book it holds lists empty.
- Not covered: iCloud answers an unknown collection 400 `invalid collection ID URI`, which keeps its refusal; a CardDAV `card add` into an unknown collection and a JMAP `card add` naming one were not observed live and keep the server's answer.
