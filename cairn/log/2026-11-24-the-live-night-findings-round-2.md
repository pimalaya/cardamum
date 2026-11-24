---
cairn: log
change: the-live-night-findings
date: 2026-11-24
---

# The live night findings, D3, D6 and D7

Three more items of cairn/changes/the-live-night-findings, the second round of 2026-11-24. Evidence: .lanes/live/journal.md, the lines of lane E2.

- **D3, a People group delete.** Owner ruling of 2026-11-24: the group delete keeps its contacts (`deleteContacts=false`, unchanged), a contact sitting in other groups too. The help of `addressbook delete` said it deletes "every card it contains"; it and the People `addressbook.delete` detail now say the contacts stay. Pinned by `capability::tests::a_people_group_delete_says_the_contacts_stay`.
- **D6, `jmap address-book update`.** It printed the AddressBook of the `/set` `updated` value when the server answered an object, which states only what the server changed beyond the update (RFC 8620 §5.3), io-jmap filling the rest with defaults (an `{}` answer printed `id: null`, `name: null`, every right `false`). It now prints `IdReceipt`, `{"id"}`, whatever the answer, and its JSON Schema follows; `JmapAddressBookOutput` is gone. Stalwart answers `null`, the receipt path already, so the bug was reproduced scripted. Pinned by `jmap::address_book::update::tests::an_update_prints_the_id_alone`. Live on Stalwart: `jmap address-book update o -n lane-e2-d6-again` prints `{"id":"o"}`.
- **D7, JMAP `addressbook.delete`.** Declared `full`. Read-only on Fastmail: its one AddressBook, Personal, states `myRights.mayDelete: false`, and the account may create no other (`mayCreateAddressBook: false`), so no delete can succeed there. It is now `server`, the detail naming `myRights.mayDelete: false`; the server's refusal is kept, nothing read first. Pinned by `capability::tests::a_jmap_address_book_delete_is_the_server_s`.

Each pinning test failed before its fix. Gate: `cargo fmt --check` clean; `cargo clippy --locked --all-targets -- -D warnings` clean over the 11 feature sets of .github/workflows/tests.yml; `cargo test --locked --all-features` 385 passed, 19 ignored; the two single-backend CI steps green (151 and 10 passed).

Capabilities moved: backends (a People group delete keeps its contacts, a JMAP AddressBook may forbid its deletion), commands (the `jmap address-book update` receipt).
