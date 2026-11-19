---
cairn: log
change: the-store-rules-io-pimdir-states
date: 2026-11-19
---

# The store rules io-pimdir states

cardamum moved onto io-pimdir 0.18.0 and pimalaya-cli 0.7.0 (ical-rs 0.7.2 through io-pimdir), crates.io only, and dropped its copies of the store rules io-pimdir now exposes (pimdir audit C5):

- `is_card_kind` and its test removed; `PimdirBackend::collections` keeps a collection whose `collection::media_type(kind)` is `text/vcard`. Per the owner's ruling, an undeclared collection (empty kind) is no address book, where cardamum counted it as one.
- `resolve_account` reads `PimdirReader::accounts_of`, keeping its messages and codes; a configured account on a store holding no collection is still accepted, now tested and specced.
- `scope` is `PimdirQuery::scoped(Some(PimdirQueryKind::Contact), account, addressbook)` after `known_collection`; `PimdirCardCoverage` reads `PimdirSearchCoverage::unread_counted()`.

Tests: `an_undeclared_collection_is_no_address_book`, failing on the old code (`store-account-ambiguous`, `old, work`); `a_configured_account_opens_a_store_holding_nothing`. Capability **backends** moved.
