---
cairn: change
id: the-store-rules-io-pimdir-states
status: landed
created: 2026-11-19
---

# The store rules io-pimdir states

## Why

pimdir audit-2026-11-12 C5: cardamum restated store rules io-pimdir owns, and the three CLIs' copies differed. io-pimdir 0.18.0 exposes them (io-pimdir cairn/changes/the-store-rules-the-clis-restate), and the owner ruled on 2026-11-19 that a configured account is accepted on a store holding no collection yet and that an empty `kind` belongs to no application's kind.

## What

- `is_card_kind` and its test go: an address book is a collection whose `collection::media_type(kind)` is `text/vcard`; the empty kind no longer counts.
- `resolve_account` reads `PimdirReader::accounts_of(None)` and `accounts_of(Some("text/vcard"))` in place of its own closure, keeping its messages and codes; a configured account on a store holding nothing stays accepted, now specced and tested.
- `scope` becomes `PimdirQuery::scoped(Some(PimdirQueryKind::Contact), account, addressbook)` after its `known_collection` check.
- `PimdirCardCoverage` takes `bodiless`, `missing` and `unparseable` from `PimdirSearchCoverage::unread_counted()`.

Users see one change: an undeclared collection is no address book, and its account no longer makes the derived account ambiguous.
