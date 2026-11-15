---
cairn: log
change: an-unknown-addressbook-is-named
date: 2026-11-15
---

# An unknown addressbook is named

`card list` naming an address book CardDAV, JMAP or Graph does not hold fails `addressbook-not-found`, ``Addressbook `<id>` not found``, the server's answer in `sources`, where it printed the bare refusal uncoded. Task C32 of cli-alignment-2026-11-15, live findings D1 and J2 and the Graph `card list` row. Capability **backends** moved (its refusal table's `addressbook-not-found` row too).

- **src/error.rs** `addressbook_not_found`, used by People's `group_error` too.
- **src/carddav/backend.rs** `addressbook_error`: 404 or 410 on `card list`, `addressbook update`, `addressbook delete`. **src/msgraph/backend.rs** `folder_error`: 404 or `ErrorInvalidIdMalformed` on `card list`, `card add`, `addressbook update`, `addressbook delete`. **src/jmap/backend.rs** `holds_addressbook`: a query failing or empty asks `AddressBook/get` once.
- **Open**: iCloud's 400 `invalid collection ID URI` keeps its refusal; a CardDAV or JMAP `card add` into an unknown book was not observed live and is unchanged.
- **Evidence**: `an_unknown_address_book_is_not_found` (CardDAV 404 list and delete; JMAP refused and empty queries over a `notFound` book, two requests, and an empty book it holds listing empty), `an_unknown_folder_is_not_found` (Graph), failing on the code before.

CHANGELOG (Fixed). Spec: backends ("A network backend names an unknown address book" added, "A server's refusal carries its code").
