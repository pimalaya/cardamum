---
cairn: tasks
change: the-store-rules-io-pimdir-states
---

# Tasks

- [x] io-pimdir 0.18.0, pimalaya-cli 0.7.0 and ical-rs 0.7.2 (through io-pimdir) in Cargo.lock.
- [x] `an_undeclared_collection_is_no_address_book` (src/pimdir/backend.rs), failing on the old code (`store-account-ambiguous` naming `old, work`), and `a_configured_account_opens_a_store_holding_nothing`.
- [x] `is_card_kind` and its asserts removed, `collections` filtering on `collection::media_type`.
- [x] `resolve_account` on `PimdirReader::accounts_of`.
- [x] `scope` on `PimdirQuery::scoped`, `PimdirCardCoverage` on `unread_counted()`.
- [x] backends spec folded, CHANGELOG, log.
