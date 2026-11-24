---
cairn: log
change: the-live-night-findings
date: 2026-11-24
---

# The live night findings, D2, D4 and D5

Three items of cairn/changes/the-live-night-findings, met on the live night of 2026-11-23. Evidence: .lanes/live/journal.md, the lines of lane E.

- **D2, JMAP address book writes.** The table declared `addressbook.create` and `addressbook.update` `full`, while Fastmail's session states `mayCreateAddressBook: false` and refuses both. `account capability` connects nowhere, so the table now says `server` for both (the detail naming the flag and Fastmail), and `addressbook create` reads the flag from the session it holds: `false` fails coded `permission-denied` before any `AddressBook/set`, as calendula reads `mayCreateCalendar`; an account stating nothing is left to the server. Pinned by `capability::tests::jmap_address_book_writes_are_the_server_s` and `jmap::backend::tests::an_account_that_may_not_create_is_refused_before_the_set`. Live on Fastmail: `addressbook create live-2026-11-24-cardamum-e3` exits 1 coded `permission-denied`, nothing created.
- **D4, `jmap address-book create`.** It printed the AddressBook of the `/set` reply, which states only the properties the server chose, io-jmap filling the rest with defaults (`name: null`, every right `false`). It prints `IdReceipt`, `{"id"}`, and its JSON Schema follows. Pinned by `jmap::address_book::create::tests::a_create_prints_the_id_alone`. Live on Stalwart: `{"id":"m"}`, while `jmap address-book get` shows that book named and with every right.
- **D5, `vdir rename` onto an existing collection.** It failed with the raw "Directory not empty (os error 39)", or replaced an empty directory silently. A new id that is one directory name the home holds (any entry) is refused coded `addressbook-duplicate`, ``Cannot rename collection `<id>` to `<new-id>`: it exists already``, nothing moved. Pinned by tests/duplicate.rs `a_collection_renamed_onto_another_is_a_duplicate`.

Each pinning test failed before its fix. Gate: `cargo fmt --check` clean; `cargo clippy --locked --all-targets -- -D warnings` clean over the 11 feature sets of .github/workflows/tests.yml; `cargo test --all-features` 382 passed, 19 ignored; the two single-backend CI steps green (149 and 10 passed).

Capabilities moved: backends (a collection renamed onto another refused, a JMAP account may forbid address book writes, two code rows), commands (the `jmap address-book create` receipt, the `addressbook-duplicate` row).
